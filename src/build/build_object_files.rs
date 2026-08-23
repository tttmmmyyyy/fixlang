use crate::{
    ast::{
        export_statement::ExportStatement,
        expr::ExprNode,
        name::FullName,
        program::{Program, Symbol, TypeEnv},
        types::TypeNode,
    },
    build::compile_unit::CompileUnit,
    configuration::{Configuration, OutputFileType},
    constants::{
        C_ENTRY_POINT_NAME, DOT_FIXLANG, GLOBAL_VAR_NAME_ARGC, GLOBAL_VAR_NAME_ARGV,
        UNITS_CACHE_PATH,
    },
    error::{panic_with_msg, Errors},
    ffi::c_entry_point_signature,
    fixstd::{
        builtin::run_io_or_ios_runner,
        runtime::{self, BuildMode},
    },
    generator::{enum_attribute_kind_id, module_functions, Generator},
    hash::HashSource,
    misc::{info_msg, join_compiler_threads, spawn_compiler_thread, warn_msg, Map, Set},
    optimization::optimization,
    rc_ir::{
        ast::RcProgram,
        borrow::{borrow_ify, cancel, param_ownership_shapes, split_rc_units},
        dead_code_elim, locality,
        lower::lower_program,
        print::{program_to_string_annotated, Annotations},
        provenance::analyze_program,
        rc_insert::insert_rc,
        simplify::simplify,
        unique_check_elim, validate,
    },
    tool::stopwatch::StopWatch,
};
use inkwell::{
    attributes::AttributeLoc,
    context::Context,
    module::Module,
    passes::PassBuilderOptions,
    targets::{CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine},
    values::BasicValue,
    OptimizationLevel,
};
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use std::{
    fmt::Display,
    fs::{self, create_dir_all, File},
    mem,
    path::{Path, PathBuf},
    sync::Arc,
};

/// What a build produced, as `build_object_files` reports it.
#[derive(Clone, Serialize, Deserialize)]
pub struct BuildObjFilesResult {
    /// The object files generated, one per compilation unit.
    pub obj_paths: Vec<PathBuf>,
}

/// The names through which code generation reaches this program's functions and globals from outside
/// it. `dead_code_elim::eliminate_unreachable` keeps them and everything they reach.
///
/// Separated compilation publishes every symbol of a unit, so that another unit can call it: a
/// funptr symbol becomes a function of external linkage, and any other symbol becomes a value
/// reached through an accessor of external linkage (`Generator::declare_lambda_function`,
/// `Configuration::external_if_separated`). Which of them the rest of the program uses is not
/// knowable from the unit, so all of them are roots — decided by the rule that decides the linkage.
///
/// Compiled as one unit, every function and global is internal, and the C world enters the program
/// in exactly two places: the entry point `build_main_function` runs, and the values
/// `build_exported_c_functions` publishes under their C names — which together are what
/// `Program::root_value_names` names.
fn reachability_roots(
    symbols: &[Symbol],
    root_value_names: &[FullName],
    config: &Configuration,
) -> Set<FullName> {
    if config.enable_separated_compilation() {
        return symbols.iter().map(|sym| sym.name.clone()).collect();
    }
    // Compiled as one unit, `symbols` is the whole program, so every root value names one of its
    // symbols. The walk starts at these names alone: one that names no definition reaches nothing,
    // and the code behind that root value is dropped.
    let symbol_names: Set<&FullName> = symbols.iter().map(|sym| &sym.name).collect();
    for root_value_name in root_value_names {
        assert!(
            symbol_names.contains(root_value_name),
            "the root value `{}` names no symbol of the program",
            root_value_name.to_string()
        );
    }
    root_value_names.iter().cloned().collect()
}

/// Lower `symbols` to the RC IR and insert reference counting. Reference counting is what the
/// lowered program needs to run at all; the optimizations over it are separate
/// (`optimize_rc_program`).
///
/// # Arguments
/// * `symbols` — the set to lower and generate code for: one compilation unit, or the whole program.
/// * `global_types` — the type of a global that a lowered function references as an LLVM operand.
///   Under separated compilation such a global may be defined in another unit, so this covers the
///   whole program.
/// * `roots` — the names code generation reaches the lowered program through from outside it
///   (`reachability_roots`).
fn lower_and_insert_rc(
    type_env: &TypeEnv,
    symbols: &[Symbol],
    global_types: &Map<FullName, Arc<TypeNode>>,
    roots: Set<FullName>,
    config: &Configuration,
) -> RcProgram {
    let mut prog = lower_program(type_env, symbols, global_types, roots);
    // Simplify the plain lowered term (case-of-known-constructor / case-of-case) before reference
    // counting is inserted, so `insert_rc` computes optimal counts over the already-simplified code.
    if config.enable_simplify() {
        simplify(&mut prog, config);
    }
    insert_rc(&mut prog, type_env);
    prog
}

/// Normalize reference counting to unit granularity, then — at `Max` and above — optimize: borrow
/// read-only parameters, cancel the reference counting a borrow makes net-zero, and specialize
/// functions by input uniqueness to elide unique checks. Borrow-ification records each version's
/// borrowed parameters on the functions (`RcFunc::borrowed_units`), which `param_ownership_shapes`
/// reads back as the owned complement.
fn optimize_rc_program(
    mut prog: RcProgram,
    type_env: &TypeEnv,
    global_types: &Map<FullName, Arc<TypeNode>>,
    config: &Configuration,
) -> RcProgram {
    // The whole program's symbol names, for the debug-only validator to recognize global references
    // (a unit may reference a symbol another unit defines). Built only when the validator runs.
    let symbol_names: Set<FullName> = if config.develop_mode {
        global_types.keys().cloned().collect()
    } else {
        Set::default()
    };
    let validate = |prog: &RcProgram, stage: &str| {
        if config.develop_mode {
            validate::validate(prog, &symbol_names, type_env, stage);
        }
    };
    // Drop what nothing reaches, after `stage` and before the pass below it runs. Each pass this
    // guards sends a call to a new version of its callee, leaving the version it moved off with one
    // caller fewer, and the last such call leaves it with none — along with every function only that
    // version called. Pruning between the passes is what keeps the one below from cloning, analyzing
    // and generating code for functions no execution reaches.
    //
    // The levels below these passes reroute no call, and lowering names every function it lifts from
    // the symbol it lifted it out of, so a program there holds nothing to drop.
    let prune = |prog: &mut RcProgram, stage: &str| {
        dead_code_elim::eliminate_unreachable(prog);
        validate(prog, stage);
    };

    validate(&prog, "after insert_rc");
    split_rc_units(&mut prog, type_env);
    validate(&prog, "after split_rc_units");
    if config.enable_borrow_optimization() {
        prog = borrow_ify(&prog, type_env);
        validate(&prog, "after borrow_ify");
        prog = cancel(&prog, type_env);
        validate(&prog, "after cancel");
        prune(&mut prog, "after dce following cancel");
        prog = unique_check_elim::specialize(&prog, type_env);
        validate(&prog, "after specialize");
        prune(&mut prog, "after dce following specialize");
        // Locality inference rests on nothing moving a live object out of the local state, which a
        // threaded build breaks: `mark_threaded` marks an object every existing binding to it still
        // reaches. A threaded build keeps the runtime dispatch everywhere.
        if !config.threaded {
            prog = locality::specialize(&prog, type_env);
            validate(&prog, "after locality");
            prune(&mut prog, "after dce following locality");
        }
    }
    prog
}

/// Write the `stage` (`pre` or `post` optimization) RC IR of the module selected by `filter` to a
/// file under `.fixlang/`: `rc_ir.<module>.<stage>.txt`, or `rc_ir.<stage>.txt` for `all`. Behind
/// `--emit-rc-ir`, for compiler development.
///
/// # Arguments
/// * `rc_program` — the whole program at that stage. The module filter is applied here, on the RC
///   IR, so the dumped functions carry the whole-program context code generation compiles.
fn dump_rc_ir(
    rc_program: &RcProgram,
    type_env: &TypeEnv,
    filter: &str,
    stage: &str,
    config: &Configuration,
) {
    // Provenance and ownership are optimization-analysis outputs, so only the post-optimization dump
    // carries them; the pre-optimization dump shows the plain lowered RC IR.
    let post = stage == "post";
    let provs = post.then(|| analyze_program(rc_program, type_env).bindings);
    let param_ownerships = (post && config.enable_borrow_optimization())
        .then(|| param_ownership_shapes(rc_program, type_env));
    let ann = Annotations {
        provs: provs.as_ref(),
        param_ownerships: param_ownerships.as_ref(),
    };

    // Keep the functions and globals of the selected module. Every function name carries its source
    // module in its top namespace component — a top-level name, a `<function>::closure{N}` lambda, or a
    // clone of either. Annotations stay computed over the whole program, so a kept function's
    // provenance and ownership still resolve.
    let in_module = |name: &FullName| {
        filter == "all" || name.namespace.names.first().map(String::as_str) == Some(filter)
    };
    let selected = RcProgram {
        funcs: rc_program
            .funcs
            .iter()
            .filter(|(r, _)| in_module(&r.name))
            .map(|(r, f)| (r.clone(), f.clone()))
            .collect(),
        globals: rc_program
            .globals
            .iter()
            .filter(|g| in_module(&g.symbol))
            .cloned()
            .collect(),
        roots: rc_program.roots.clone(),
    };

    // `filter` is arbitrary command-line input, so keep only characters safe in a file name.
    let file_name = if filter == "all" {
        format!("rc_ir.{}.txt", stage)
    } else {
        let mod_name: String = filter
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || matches!(c, '.' | '-' | '_') {
                    c
                } else {
                    '_'
                }
            })
            .collect();
        format!("rc_ir.{}.{}.txt", mod_name, stage)
    };
    let path = PathBuf::from(DOT_FIXLANG).join(file_name);
    if let Err(e) = fs::write(&path, program_to_string_annotated(&selected, ann)) {
        panic_with_msg(&format!(
            "Failed to write RC IR to `{}`: {}",
            path.display(),
            e
        ));
    }
    info_msg(&format!("RC IR written to {}.", path.display()));
}

/// Dump the RC IR for inspection when `--emit-rc-ir` is given. Lower the whole program (as code
/// generation does at `Max`), then write it before and after the optimizations, filtered to the
/// requested module in each dump.
fn dump_rc_ir_stages(program: &Program, config: &Configuration) {
    let Some(filter) = &config.emit_rc_ir else {
        return;
    };
    let type_env = program.type_env();
    let all_symbols: Vec<Symbol> = program.symbols.values().cloned().collect();
    let global_types = program.global_types();
    let roots = reachability_roots(&all_symbols, &program.root_value_names(), config);
    let base = lower_and_insert_rc(&type_env, &all_symbols, &global_types, roots, config);
    dump_rc_ir(&base, &type_env, filter, "pre", config);
    let optimized = optimize_rc_program(base, &type_env, &global_types, config);
    dump_rc_ir(&optimized, &type_env, filter, "post", config);
}

/// Compile the program into object files, and return their paths for the linker.
pub fn build_object_files<'c>(
    mut program: Program,
    config: &Configuration,
) -> Result<BuildObjFilesResult, Errors> {
    let _sw = StopWatch::new("build_object_files", config.show_build_times);

    // Return cached object files if available.
    // This cache is especially effective when running "fix run" repeatedly without editing the source code.
    if !config.dumps_generated_code() {
        if let Some(cached) = load_build_object_files_cache(&program, config) {
            if config.verbose {
                info_msg("Using cached object files.");
            }
            return Ok(cached);
        }
    }

    // Run optimizations.
    optimization::run(&mut program, &config);

    // The layout validation before code generation runs on the program as elaboration left it, and
    // the optimizations that follow mint types of their own — a capture list, a punched type, the
    // pair a newtype opens into. Those reach code generation without having been validated, so in
    // development mode the program is validated again here, where the types are the ones code
    // generation will actually lay out. A report at this point names a type the compiler built, so
    // only a development build runs it.
    if config.develop_mode {
        program.validate_layouts()?;
    }

    dump_rc_ir_stages(&program, config);

    // Determine compilation units.
    let mut units = vec![];
    let mut symbols = program.symbols.values().cloned().collect::<Vec<_>>();
    symbols.sort_by(|a, b| a.name.cmp(&b.name));
    // Every unit needs the types of the whole program's globals, so build them once and share.
    let global_types = Arc::new(program.global_types());
    // The names the C world enters the program through. Taken here, before the main unit moves the
    // export statements out of the program, so that every unit is handed the same set.
    let root_value_names = Arc::new(program.root_value_names());
    {
        let module_dependency_hash = program.module_dependency_hash_map(&config);
        let module_dependency_map = program.module_dependency_map();
        let modules = program.linked_mods().iter().cloned().collect::<Vec<_>>();
        if config.enable_separated_compilation() {
            units = CompileUnit::split_symbols(
                symbols,
                &module_dependency_hash,
                &module_dependency_map,
                &config,
            );
            // Also add main compilation unit.
            // The main unit implements the entry point of exported functions.
            // Therefore, the main unit is treated as depending on all modules.
            let mut main_unit = CompileUnit::new(vec![], modules);
            main_unit.update_unit_hash(&module_dependency_hash, &config);
            units.push(main_unit);
        } else {
            // Add main compilation unit, which includes all symbols.
            let mut main_unit = CompileUnit::new(symbols, modules);
            main_unit.update_unit_hash(&module_dependency_hash, &config);
            units.push(main_unit);
        }
    }

    // Paths of object files to be linked.
    let mut obj_paths = vec![];

    // Generate object files in parallel.
    let mut threads = vec![];
    let units_count = units.len();
    for (i, unit) in units.into_iter().enumerate() {
        // The main unit is generated last.
        let is_main_unit = i == units_count - 1;

        obj_paths.push(unit.object_file_path());
        // If the object file is cached, skip the generation.
        if unit.is_cached() && !config.dumps_generated_code() {
            if config.verbose {
                info_msg(&format!(
                    "Skipping generation of object file for {}.",
                    unit.to_string()
                ));
            }
            continue;
        }
        if config.verbose {
            info_msg(&format!("Generating object file for {}.", unit.to_string()));
        }

        let global_types = global_types.clone();
        let root_value_names = root_value_names.clone();
        let config = config.clone();
        let type_env = program.type_env();

        let export_statements = if is_main_unit {
            // Export statements are only needed for the main unit.
            mem::replace(&mut program.export_statements, vec![])
        } else {
            vec![]
        };

        let entry_io_value = program.entry_io_value.clone();
        threads.push(spawn_compiler_thread(move || {
            let context = Context::create();
            let target_machine = get_target_machine(config.get_llvm_opt_level(), &config);
            let module = Generator::create_module(
                &format!("Module-{}", unit.unit_hash()),
                &context,
                &target_machine,
            );
            let mut gc = Generator::new(
                &context,
                &module,
                target_machine.get_target_data(),
                config.clone(),
                type_env,
                global_types.clone(),
            );

            // In debug mode, create debug infos.
            if config.debug_info {
                gc.create_debug_info();
            }

            // Declare runtime functions.
            runtime::build_runtime(&mut gc, BuildMode::Declare);

            // Lower this unit's symbols to the RC IR, insert reference counting, and generate their
            // LLVM. Only this unit's symbols are implemented; a symbol of another unit that this
            // one calls is declared where code generation first reaches it, from the types of the
            // program's globals the generator was given.
            let unit_symbols = unit.symbols().to_vec();
            let roots = reachability_roots(&unit_symbols, &root_value_names, &config);
            let rc_prog =
                lower_and_insert_rc(gc.type_env(), &unit_symbols, &global_types, roots, &config);
            let rc_prog = optimize_rc_program(rc_prog, gc.type_env(), &global_types, &config);
            gc.implement_rc_program(&rc_prog);

            if is_main_unit {
                // Implement runtime functions.
                runtime::build_runtime(&mut gc, BuildMode::Implement);

                // Implement exported C functions.
                build_exported_c_functions(&mut gc, &export_statements);

                // Implement the `main()` function.
                if let Some(main_expr) = entry_io_value {
                    build_main_function(&mut gc, main_expr.clone());
                }
            }

            // Every reader of this unit's globals is in the module by now.
            gc.keep_initializers_out_of_shared_accessors();

            gc.finalize_di();

            gc.assert_defined_symbols_fit_a_symbol_table();

            // Add frame-pointer attribute to all functions for better backtraces on macOS
            if config.no_elim_frame_pointers() {
                gc.add_frame_pointer_attribute_to_all_functions();
            }

            if config.emit_llvm {
                // Print LLVM-IR to file before optimization.
                emit_llvm(gc.module, &config, false);
            }

            optimize_instrument_and_verify(gc.module, &target_machine, &config);

            if config.emit_llvm {
                // Print LLVM-IR to file after optimization.
                emit_llvm(gc.module, &config, true);
            }

            write_to_object_file(gc.module, &target_machine, &unit.object_file_path());
        }));
    }
    join_compiler_threads(threads);

    // Save object files cache.
    let obj_files = BuildObjFilesResult { obj_paths };
    save_build_object_files_cache(&program, config, &obj_files);

    Ok(obj_files)
}

/// The object files a previous build of this program and configuration left behind, when the cache
/// records them and every one of them is still on disk.
fn load_build_object_files_cache(
    program: &Program,
    config: &Configuration,
) -> Option<BuildObjFilesResult> {
    let hash = build_object_files_cache_hash_or_warn(program, config)?;
    let cache_path = build_object_files_cache_path(&hash);
    if !Path::new(&cache_path).exists() {
        return None;
    }
    let file = cache_step_or_warn(
        File::open(&cache_path),
        &format!("Failed to open object files cache \"{}\"", cache_path),
    )?;
    let cache: BuildObjFilesResult = cache_step_or_warn(
        serde_json::from_reader(file),
        &format!("Failed to read object files cache \"{}\"", cache_path),
    )?;
    // Check all files in the cache exist.
    for path in &cache.obj_paths {
        if !path.exists() {
            return None;
        }
    }
    Some(cache)
}

/// Record the object files a build produced under the hash of the program and configuration it
/// built, so that an identical build reuses them.
fn save_build_object_files_cache(
    program: &Program,
    config: &Configuration,
    obj_files: &BuildObjFilesResult,
) {
    let Some(hash) = build_object_files_cache_hash_or_warn(program, config) else {
        return;
    };
    let Some(()) = cache_step_or_warn(
        create_dir_all(UNITS_CACHE_PATH),
        "Failed to create directory for object files cache",
    ) else {
        return;
    };
    let cache_path = build_object_files_cache_path(&hash);
    let Some(file) = cache_step_or_warn(
        File::create(&cache_path),
        &format!("Failed to create object files cache \"{}\"", cache_path),
    ) else {
        return;
    };
    cache_step_or_warn(
        serde_json::to_writer_pretty(file, obj_files),
        &format!("Failed to write object files cache \"{}\"", cache_path),
    );
}

/// The value `result` carries, or `None` after warning with `failure_msg` and the error behind it.
/// The object files cache is an optimization, so a step of reading or writing it that fails gives up
/// on the cache and lets the build go on.
fn cache_step_or_warn<T, E: Display>(result: Result<T, E>, failure_msg: &str) -> Option<T> {
    match result {
        Ok(value) => Some(value),
        Err(e) => {
            warn_msg(&format!("{}: {}.", failure_msg, e));
            None
        }
    }
}

/// The file the object files a build produced are recorded in, named by the hash of the build. The
/// reader and the writer of the cache take the path from here, so they name one file.
fn build_object_files_cache_path(hash: &str) -> String {
    format!("{}/{}.json", UNITS_CACHE_PATH, hash)
}

/// The hash that names the object files cache of a build: it covers the configuration options that
/// bear on code generation together with every source every module is made of, so two builds share
/// a hash exactly when they would produce the same object files.
fn build_object_files_cache_hash(
    program: &Program,
    config: &Configuration,
) -> Result<String, Errors> {
    let mut hash_source = HashSource::default();
    hash_source.push_text(&config.object_generation_hash());
    // What this cache holds is the object files of a whole build, and how many of them there are is
    // decided by how many symbols one compilation unit holds. A unit's own object file is named by
    // the symbols it holds (`CompileUnit::update_unit_hash`), so a build that divides itself
    // differently still reuses each unit whose symbols it leaves together.
    hash_source.push_text(&config.cu_size.to_string());
    for mi in &program.modules {
        hash_source.push_text(&mi.source.input.hash()?);
    }
    Ok(hash_source.finish())
}

/// The hash naming a build's object files cache, or `None` after warning that it could not be
/// calculated. The hash is the cache's file name, so a build that lacks it goes on without the
/// cache.
fn build_object_files_cache_hash_or_warn(
    program: &Program,
    config: &Configuration,
) -> Option<String> {
    cache_step_or_warn(
        build_object_files_cache_hash(program, config),
        "Failed to calculate hash of object files cache",
    )
}

/// The LLVM target machine to compile for: the host's CPU with the features it supports, minus the
/// ones the configuration disables, generating code at `opt_level`. A dynamic library is compiled
/// position-independent.
pub(crate) fn get_target_machine(
    opt_level: OptimizationLevel,
    config: &Configuration,
) -> TargetMachine {
    let _native = Target::initialize_native(&InitializationConfig::default())
        .map_err(|e| panic_with_msg(&format!("failed to initialize native: {}", e)))
        .unwrap();
    let triple = TargetMachine::get_default_triple();
    let target = Target::from_triple(&triple)
        .map_err(|e| {
            panic_with_msg(&format!("failed to create target: {}", e));
        })
        .unwrap();
    let reloc_mode = if matches!(config.output_file_type, OutputFileType::DynamicLibrary) {
        RelocMode::PIC
    } else {
        RelocMode::Default
    };
    let target_machine = target.create_target_machine(
        &triple,
        &config.host_cpu.name,
        &config.target_cpu_features(),
        opt_level,
        reloc_mode,
        CodeModel::Default,
    );
    match target_machine {
        Some(tm) => tm,
        None => panic_with_msg("Failed to create target machine."),
    }
}

/// Compile `module` into an object file at `obj_path`, creating the containing directory. The code
/// goes to a uniquely named temporary file that is renamed into place, so `obj_path` exists only
/// once it holds a complete object.
fn write_to_object_file<'c>(module: &Module<'c>, target_machine: &TargetMachine, obj_path: &Path) {
    // Create directory if it doesn't exist.
    let dir_path = obj_path.parent().unwrap();
    match fs::create_dir_all(dir_path) {
        Err(e) => {
            panic_with_msg(&format!(
                "Failed to create directory \"{}\": {}",
                dir_path.to_string_lossy().to_string(),
                e
            ));
        }
        Ok(_) => {}
    }
    // Write to a temporary file.
    let tmp_file_path = obj_path.with_extension(thread_rng().gen::<u64>().to_string() + ".tmp");
    target_machine
        .write_to_file(&module, FileType::Object, &tmp_file_path)
        .map_err(|e| {
            panic_with_msg(&format!(
                "Failed to write to file \"{}\": {}",
                obj_path.to_string_lossy().to_string(),
                e
            ))
        })
        .unwrap();

    // Rename the temporary file to the final file.
    match fs::rename(&tmp_file_path, obj_path) {
        Err(e) => {
            panic_with_msg(&format!(
                "Failed to rename \"{}\" to \"{}\": {}",
                tmp_file_path.to_string_lossy().to_string(),
                obj_path.to_string_lossy().to_string(),
                e
            ));
        }
        Ok(_) => {}
    }
}

/// Write `module`'s LLVM-IR to a text file whose name records the module and whether the LLVM
/// optimization pipeline has already run over it.
fn emit_llvm<'c>(module: &Module<'c>, config: &Configuration, optimized: bool) {
    let unit_name = module.get_name().to_str().unwrap();
    let path = config.get_output_llvm_ir_path(optimized, unit_name);
    if let Err(e) = module.print_to_file(path.clone()) {
        panic_with_msg(&format!("Failed to emit LLVM-IR: {}", e.to_string()));
    }
}

/// Hands each pass-pipeline string to LLVM's pass builder in turn, aborting the compilation if LLVM
/// rejects one.
fn run_passes_or_panic(
    module: &Module,
    passes: &[impl AsRef<str>],
    target_machine: &TargetMachine,
) {
    for pass in passes {
        let pass = pass.as_ref();
        if let Err(e) = module.run_passes(pass, target_machine, PassBuilderOptions::create()) {
            panic_with_msg(&format!(
                "Failed to run pass \"{}\": {}",
                pass,
                e.to_string()
            ));
        }
    }
}

/// Verifies `module`, runs the LLVM optimization pipeline the configuration selects over it,
/// instruments it for the configured sanitizer, then verifies it again. A module LLVM rejects, or a
/// pipeline it cannot build, aborts the compilation.
fn optimize_instrument_and_verify<'c>(
    module: &Module<'c>,
    target_machine: &TargetMachine,
    config: &Configuration,
) {
    run_passes_or_panic(module, &["verify"], target_machine);
    run_passes_or_panic(module, &config.llvm_passes(), target_machine);
    instrument_for_sanitizer(module, target_machine, config);
    run_passes_or_panic(module, &["verify"], target_machine);
}

/// Instruments `module` for the configured sanitizer, so that its runtime sees the program's memory
/// accesses.
///
/// The instrumentation runs after the optimization pipeline, which is where clang puts it: an
/// access the optimizer removes is one the program never makes.
///
/// It sits outside `Configuration::llvm_passes` because `--llvm-passes-file` replaces what that
/// returns. Were the instrumentation part of it, a build could drop the instrumentation while
/// still reporting itself as sanitized.
fn instrument_for_sanitizer<'c>(
    module: &Module<'c>,
    target_machine: &TargetMachine,
    config: &Configuration,
) {
    let Some((attribute_name, passes)) = config.sanitizer.instrumentation() else {
        return;
    };
    add_attribute_to_defined_functions(module, attribute_name);
    run_passes_or_panic(module, passes, target_machine);
}

/// Gives every function of `module` that has a body the attribute named `attribute_name`.
///
/// The instrumentation passes rewrite the functions carrying their attribute and leave the rest
/// alone, which is how clang lets a translation unit opt out. Nothing else here sets it, so without
/// this the passes would run over the module and change nothing.
fn add_attribute_to_defined_functions<'c>(module: &Module<'c>, attribute_name: &str) {
    let attribute = module
        .get_context()
        .create_enum_attribute(enum_attribute_kind_id(attribute_name), 0);
    for function in module_functions(module) {
        if function.count_basic_blocks() > 0 {
            function.add_attribute(AttributeLoc::Function, attribute);
        }
    }
}

/// Emit the C entry point of each `FFI_EXPORT` statement.
fn build_exported_c_functions<'c, 'm>(
    gc: &mut Generator<'c, 'm>,
    export_stmts: &[ExportStatement],
) {
    for export_stmt in export_stmts {
        export_stmt.implement(gc);
    }
}

/// Implement the C `main` function of the program: store `argc` and `argv` into the global
/// variables the runtime reads them from, run the `IO ()` action `main_expr` refers to, and return
/// 0.
///
/// The body goes onto the declaration an `FFI_CALL` of `main` has left, where a program calls its
/// own entry point. `Program::validate_c_function_calls` has held that call to
/// `c_entry_point_signature`, so the declaration found here is the one this function builds.
fn build_main_function<'c, 'm>(gc: &mut Generator<'c, 'm>, main_expr: Arc<ExprNode>) {
    let main_function =
        c_entry_point_signature().get_or_declare_in_module(&C_ENTRY_POINT_NAME.to_string(), gc);
    assert_eq!(
        main_function.count_basic_blocks(),
        0,
        "the entry point has one definition"
    );
    let entry_bb = gc.context.append_basic_block(main_function, "entry");
    gc.builder().position_at_end(entry_bb);

    // Save argc and argv to global variables.
    for (i, global_var_name) in [GLOBAL_VAR_NAME_ARGC, GLOBAL_VAR_NAME_ARGV]
        .iter()
        .enumerate()
    {
        let arg_val = main_function.get_nth_param(i as u32).unwrap();
        let gv_ptr = gc
            .module
            .get_global(global_var_name)
            .unwrap()
            .as_basic_value_enum()
            .into_pointer_value();
        gc.builder().build_store(gv_ptr, arg_val).unwrap();
    }

    // Run the main IO action. `main_expr` is a reference to the instantiated `main` symbol (see
    // `instantiate_exported_value`), which the RC-IR back end has already implemented; materialize
    // that symbol's object here.
    let main_name = main_expr.get_var().name.clone();
    let main_obj = gc.get_scoped_obj(&main_name); // A value of type `IO ()`.
    run_io_or_ios_runner(gc, &main_obj);

    // Return main function.
    gc.builder()
        .build_return(Some(&gc.context.i32_type().const_int(0, false)))
        .unwrap();
}
