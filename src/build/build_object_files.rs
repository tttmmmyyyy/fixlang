use crate::{
    ast::{
        export_statement::ExportStatement,
        expr::ExprNode,
        name::FullName,
        program::{Program, Symbol, TypeEnv},
        types::TypeNode,
    },
    build::divide_program::{
        divide_among_units, divide_into_units, generated_code_hash, DividedProgram,
    },
    configuration::{Configuration, OutputFileType},
    constants::{
        C_ENTRY_POINT_NAME, DOT_FIXLANG, GLOBAL_VAR_NAME_ARGC, GLOBAL_VAR_NAME_ARGV,
        UNITS_CACHE_PATH, UNIT_MODULE_NAME_PREFIX,
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
        codegen::keep_initializers_out_of_shared_accessors,
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
    /// The object files the linker is to put together, one per compilation unit.
    pub obj_paths: Vec<PathBuf>,
}

/// Lower `symbols` to the RC IR and insert reference counting. Reference counting is what the
/// lowered program needs to run at all; the optimizations over it are separate
/// (`optimize_rc_program`).
///
/// # Arguments
/// * `symbols` — the set to lower and generate code for: one compilation unit, or the whole program.
/// * `global_types` — the type of a global that a lowered function references as an LLVM operand.
///   Such a global may be defined in another unit, so this covers the whole program.
/// * `roots` — the names code generation reaches the lowered program through from outside it.
// PROOF: (P-insert) (dev-docs/proof/rc_ir/borrow-cancel)
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
// PROOF: D/A, P15, P16, P17, P18, P18c, P19, P20, P21, P22, P23, P24, P26, (P-insert), T (dev-docs/proof/rc_ir/borrow-cancel)
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
        prog = borrow_ify(&prog, type_env, config.develop_mode);
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

/// Dump the RC IR for inspection when `--emit-rc-ir` is given. Lower the whole program, then write
/// it before and after the optimizations, filtered to the requested module in each dump.
///
/// The build lowers and optimizes the same whole program and divides what the optimizations leave
/// among the compilation units, so this dump is what every unit's code is generated from.
fn dump_rc_ir_stages(program: &Program, config: &Configuration) {
    let Some(filter) = &config.emit_rc_ir else {
        return;
    };
    let type_env = program.type_env();
    let all_symbols: Vec<Symbol> = program.symbols.values().cloned().collect();
    let global_types = program.global_types();
    // The whole program is lowered here, and the C world enters it through its root values alone.
    let roots = program.root_value_names().into_iter().collect();
    let base = lower_and_insert_rc(&type_env, &all_symbols, &global_types, roots, config);
    dump_rc_ir(&base, &type_env, filter, "pre", config);
    let optimized = optimize_rc_program(base, &type_env, &global_types, config);
    dump_rc_ir(&optimized, &type_env, filter, "post", config);
}

/// Compile the program into object files, and return their paths for the linker.
// PROOF: D/A, P18c, P19, P20, P21, P22, P23, P24 (dev-docs/proof/rc_ir/borrow-cancel)
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

    let type_env = program.type_env();
    // Every unit needs the types of the whole program's globals, so build them once and share.
    let global_types = Arc::new(program.global_types());

    // The RC IR is built and optimized over the whole program, so that a pass sees every call of
    // every function it rewrites. The compilation units are cut out of what the optimizations
    // leave, so that a unit is a set of the entries whose code it generates.
    let mut symbols: Vec<Symbol> = program.symbols.values().cloned().collect();
    symbols.sort_by(|a, b| a.name.cmp(&b.name));
    let root_value_names: Set<FullName> = program.root_value_names().into_iter().collect();
    let rc_prog = lower_and_insert_rc(
        &type_env,
        &symbols,
        &global_types,
        root_value_names.clone(),
        config,
    );
    let rc_prog = optimize_rc_program(rc_prog, &type_env, &global_types, config);
    let mut units = divide_into_units(&rc_prog, config);
    let division = divide_among_units(rc_prog, &units, &global_types, root_value_names);

    // A unit is named by the code it generates, which is what its object file is cached under. The
    // main unit is the last, and it is the one that builds the entry point and the exported C
    // functions. Its digest reads the export statements here, while the program still holds them:
    // the loop below hands them to the main unit's thread.
    let last_unit_index = units.len() - 1;
    for (index, unit) in units.iter_mut().enumerate() {
        let program_for_the_entry = (index == last_unit_index).then_some(&program);
        let hash = generated_code_hash(
            unit,
            index,
            &division,
            program_for_the_entry,
            &type_env,
            config,
        );
        unit.set_unit_hash(hash);
    }
    let DividedProgram {
        mut unit_programs,
        published,
        global_types,
        imported,
        published_here: _,
        shared_globals,
    } = division;

    // The object file each unit's generated code goes into, which is what the build hands the
    // linker.
    let unit_paths = units
        .iter()
        .map(|unit| unit.object_file_path())
        .collect::<Vec<_>>();
    let obj_files = BuildObjFilesResult {
        obj_paths: unit_paths.clone(),
    };

    // Generate the code of each unit in parallel.
    let mut threads = vec![];
    let units_count = units.len();
    for (i, unit) in units.into_iter().enumerate() {
        // The main unit is generated last.
        let is_main_unit = i == units_count - 1;

        let unit_path = unit_paths[i].clone();
        // If the unit's generated code is cached, skip the generation.
        if unit_path.exists() && !config.dumps_generated_code() {
            if config.verbose {
                info_msg(&format!("Skipping generation of code for {}.", unit));
            }
            continue;
        }
        if config.verbose {
            info_msg(&format!("Generating code for {}.", unit));
        }

        let global_types = global_types.clone();
        let published = published.clone();
        let imported_here = Arc::new(imported[i].clone());
        let shared_globals = shared_globals.clone();
        let config = config.clone();
        let type_env = program.type_env();
        let unit_program = mem::take(&mut unit_programs[i]);

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
                &format!("{}{}", UNIT_MODULE_NAME_PREFIX, unit.unit_hash()),
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
                published.clone(),
                imported_here.clone(),
                shared_globals,
            );

            // In debug mode, create debug infos.
            if config.debug_info {
                gc.create_debug_info();
            }

            // Declare runtime functions.
            runtime::build_runtime(&mut gc, BuildMode::Declare);

            // Generate this unit's slice of the program's RC IR. A function of another unit that
            // this one calls is declared where code generation first reaches it, from the types of
            // the program's globals the generator was given.
            gc.implement_rc_program(&unit_program);

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

            gc.finalize_di();

            gc.assert_defined_symbols_fit_a_symbol_table();

            // Add frame-pointer attribute to all functions for better backtraces on macOS
            if config.no_elim_frame_pointers() {
                gc.add_frame_pointer_attribute_to_all_functions();
            }

            // This module is the one the optimization runs over, so it holds every reader of every
            // global it defines.
            keep_initializers_out_of_shared_accessors(gc.module, &config);

            if config.emit_llvm {
                // Print LLVM-IR to file before optimization.
                emit_llvm(gc.module, &config, false);
            }

            optimize_instrument_and_verify(gc.module, &target_machine, &config);

            if config.emit_llvm {
                // Print LLVM-IR to file after optimization.
                emit_llvm(gc.module, &config, true);
            }

            write_to_object_file(gc.module, &target_machine, &unit_path);
        }));
    }
    join_compiler_threads(threads);

    // Save object files cache.
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
    // decided by how many entries one compilation unit holds. A unit's own object file is named by
    // the code it generates (`divide_program::generated_code_hash`), so a build that divides itself
    // differently still reuses each unit whose code it leaves as it was.
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

/// Compile `module` into an object file at `obj_path`.
fn write_to_object_file<'c>(module: &Module<'c>, target_machine: &TargetMachine, obj_path: &Path) {
    write_through_temporary_file(obj_path, |path| {
        target_machine
            .write_to_file(module, FileType::Object, path)
            .map_err(|e| e.to_string())
    });
}

/// Write a file at `path` through a uniquely named temporary file in the same directory that is
/// renamed into place, so that `path` exists only once it holds the whole of what was written.
/// Creates the containing directory. A failure of any step aborts the compilation.
///
/// # Arguments
/// * `write` — writes the content to the temporary path it is handed.
fn write_through_temporary_file(path: &Path, write: impl FnOnce(&Path) -> Result<(), String>) {
    let dir_path = path.parent().unwrap();
    if let Err(e) = create_dir_all(dir_path) {
        panic_with_msg(&format!(
            "Failed to create directory \"{}\": {}",
            dir_path.to_string_lossy(),
            e
        ));
    }
    let tmp_path = path.with_extension(thread_rng().gen::<u64>().to_string() + ".tmp");
    if let Err(e) = write(&tmp_path) {
        panic_with_msg(&format!(
            "Failed to write to file \"{}\": {}",
            path.to_string_lossy(),
            e
        ));
    }
    if let Err(e) = fs::rename(&tmp_path, path) {
        panic_with_msg(&format!(
            "Failed to rename \"{}\" to \"{}\": {}",
            tmp_path.to_string_lossy(),
            path.to_string_lossy(),
            e
        ));
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
// PROOF: D/A, P26, P28 (dev-docs/proof/rc_ir/borrow-cancel)
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
