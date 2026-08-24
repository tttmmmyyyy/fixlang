//! Dividing the program's RC IR among the compilation units that generate code from it.
//!
//! The RC IR is built and optimized over the whole program, so that a pass sees every call of every
//! function it rewrites. Code generation then runs per unit, so the optimized program is divided
//! here: each function and global goes to the unit holding the symbol it came from, each unit takes
//! a copy of the small functions and the accessors it reaches in another, and a name is published
//! to the linker only where a unit that has no copy of it names it.

use crate::ast::name::FullName;
use crate::ast::program::{Program, TypeEnv};
use crate::ast::types::{TyCon, TyConInfo, TypeNode};
use crate::build::compile_unit::CompileUnit;
use crate::configuration::Configuration;
use crate::hash::HashSource;
use crate::misc::{Map, Set};
use crate::parse::sourcefile::Span;
use crate::rc_ir::ast::{for_each_node, for_each_var, FuncRef, RcFunc, RcGlobalInit, RcProgram};
use crate::rc_ir::dead_code_elim::{collect_mentions, eliminate_unreachable};
use crate::rc_ir::simplify::node_count;
use serde::Serialize;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

/// The most RC IR nodes a function may hold for a unit calling it to take a copy of its own. A copy
/// lets LLVM see the body at the call; past this size the call is what the body would compile into
/// anyway, and the copy is compiled for nothing.
const IMPORTED_FUNCTION_NODE_LIMIT: u64 = 200;

/// The program's RC IR divided among the compilation units, and what a unit needs to know about the
/// others to generate code from its own slice.
pub struct DividedProgram {
    /// One unit's slice of the program, in the order of the units it was divided among.
    pub unit_programs: Vec<RcProgram>,
    /// The names a unit defines under a linkage that publishes them to the linker: the ones the C
    /// world enters the program through, and the ones another unit's code names without holding a
    /// copy of. Everything else is internal to the unit defining it, and LLVM optimizes it knowing
    /// every call it has.
    pub published: Arc<Set<FullName>>,
    /// The type of every global the program defines, the versions the optimizer synthesized
    /// included, which is what a unit declares another unit's name from.
    pub global_types: Arc<Map<FullName, Arc<TypeNode>>>,
    /// The names each unit holds a copy of rather than owning, one set per unit.
    pub imported: Vec<Set<FullName>>,
    /// The names each unit gives external linkage: the ones it defines itself and `published` holds,
    /// which is the rule `Generator::published_to_the_linker` applies as it generates the code. One
    /// set per unit.
    pub published_here: Vec<Set<FullName>>,
    /// The globals whose storage more than one unit reaches. The owning unit defines and publishes
    /// the storage, and the units holding a copy of the accessor declare it, so the value is
    /// computed once however many units read it.
    pub shared_globals: Arc<Set<FullName>>,
}

/// Divide `program` — the whole program's RC IR, optimized — among `units`.
///
/// # Arguments
/// * `global_types` — the type of every symbol of the program, which the types of the versions the
///   optimizer synthesized are added to.
/// * `root_value_names` — the values the C world enters the program through.
pub fn divide_among_units(
    program: RcProgram,
    units: &[CompileUnit],
    global_types: &Map<FullName, Arc<TypeNode>>,
    root_value_names: Set<FullName>,
) -> DividedProgram {
    // The main unit is the last, and it is the one holding no symbol of the program: it builds the
    // entry point and the exported C functions, which is where the root values are read.
    let main_unit_symbols = units
        .last()
        .expect("a program is divided into at least the main unit")
        .symbols();
    assert!(
        main_unit_symbols.is_empty(),
        "the last compilation unit holds {} symbols, and the main unit holds none",
        main_unit_symbols.len()
    );
    let unit_of = symbol_owner(units);
    let global_types = global_types_including_synthesized(&program, global_types);
    let copyable_funcs = copyable_funcs(&program);
    // Every global, so that a unit reading one another unit owns can carry a copy of its accessor.
    let all_globals: Map<FullName, RcGlobalInit> = program
        .globals
        .iter()
        .map(|global| (global.symbol.clone(), global.clone()))
        .collect();

    let mut unit_programs: Vec<RcProgram> =
        (0..units.len()).map(|_| RcProgram::default()).collect();
    for (fref, func) in program.funcs {
        let index = unit_of(&fref.name)
            .unwrap_or_else(|| panic!("no symbol owns `{}`", fref.name.to_string()));
        unit_programs[index].funcs.insert(fref, func);
    }
    for global in program.globals {
        let index = unit_of(&global.symbol)
            .unwrap_or_else(|| panic!("no symbol owns `{}`", global.symbol.to_string()));
        unit_programs[index].globals.push(global);
    }

    let (mut imported, mut shared_globals) =
        import_what_each_unit_reaches(&mut unit_programs, &copyable_funcs, &all_globals);
    give_the_main_unit_the_root_values(
        &mut unit_programs,
        &mut imported,
        &mut shared_globals,
        &all_globals,
        &root_value_names,
    );

    let published = Arc::new(published_to_the_linker(
        &unit_programs,
        &unit_of,
        &root_value_names,
    ));
    let mut published_here: Vec<Set<FullName>> = vec![];
    for (index, unit_program) in unit_programs.iter_mut().enumerate() {
        published_here.push(
            names_defined_here(unit_program)
                .filter(|name| !imported[index].contains(name) && published.contains(name))
                .cloned()
                .collect(),
        );
        // What the unit's code is reached by: the names another unit's code calls it through, the
        // values the C world enters the program through, which the entry point and the exported C
        // functions read through the accessor the unit holding them carries, and the globals whose
        // storage another unit reads, whose owner emits that storage and the function computing it.
        unit_program.roots = names_defined_here(unit_program)
            .filter(|name| {
                published_here[index].contains(*name)
                    || root_value_names.contains(*name)
                    || shared_globals.contains(*name)
            })
            .cloned()
            .collect();
        // A function every unit calling it took a copy of is left with no caller of its own, and
        // generating it writes out a body that reaches nothing the program runs.
        eliminate_unreachable(unit_program);
    }

    DividedProgram {
        unit_programs,
        published,
        global_types: Arc::new(global_types),
        imported,
        published_here,
        shared_globals: Arc::new(shared_globals),
    }
}

/// Which unit a name belongs to, by the symbol it came from.
///
/// A function lifted or cloned out of a symbol is named by extending that symbol's name — with
/// `#`-separated segments, and with the whole of it as the namespace of a lambda lifted out of it —
/// so the symbol whose name is the longest prefix of the function's is the symbol it came from.
fn symbol_owner(units: &[CompileUnit]) -> impl Fn(&FullName) -> Option<usize> {
    let mut unit_of_symbol: Map<String, usize> = Map::default();
    for (index, unit) in units.iter().enumerate() {
        for symbol in unit.symbols() {
            unit_of_symbol.insert(symbol.name.to_string(), index);
        }
    }
    // A prefix that is a symbol's name is as long as that name, so the lengths the symbols have are
    // the lengths to try, the longest first.
    let mut symbol_lengths: Vec<usize> = unit_of_symbol.keys().map(|symbol| symbol.len()).collect();
    symbol_lengths.sort_unstable_by(|a, b| b.cmp(a));
    symbol_lengths.dedup();
    move |name: &FullName| {
        let text = name.to_string();
        symbol_lengths
            .iter()
            .filter_map(|length| text.get(..*length))
            .find_map(|prefix| unit_of_symbol.get(prefix).copied())
    }
}

/// The type of every global the program defines, the versions the optimizer synthesized included.
///
/// A unit declares a name another unit defines from this, and a synthesized version is not among
/// the program's symbols, so its own type is all there is to declare it from.
fn global_types_including_synthesized(
    program: &RcProgram,
    global_types: &Map<FullName, Arc<TypeNode>>,
) -> Map<FullName, Arc<TypeNode>> {
    let mut types = global_types.clone();
    for (fref, func) in &program.funcs {
        // A closure function is reached by an indirect call through the capture the body it was
        // lifted from builds, so its name is mentioned only in that body's own unit and it is
        // declared where code generation reaches it. Naming it here would have the unit defining it
        // declare an accessor instead, since only a funptr global is the function itself.
        if func.fn_ty.is_funptr() {
            types.insert(fref.name.clone(), func.fn_ty.clone());
        }
    }
    for global in &program.globals {
        types.insert(global.symbol.clone(), global.ty.clone());
    }
    types
}

/// The functions a unit calling one may take a copy of, by name.
fn copyable_funcs(program: &RcProgram) -> Map<FullName, RcFunc> {
    program
        .funcs
        .iter()
        // A closure function is reached only through the closure value a body builds, and code
        // generation reads that function out of the module building it, so a unit that took a copy
        // of a body building one takes a copy of the function too, whatever its size.
        .filter(|(_, func)| {
            !func.fn_ty.is_funptr() || node_count(&func.body) <= IMPORTED_FUNCTION_NODE_LIMIT
        })
        .map(|(fref, func)| (fref.name.clone(), func.clone()))
        .collect()
}

/// The names `unit_program` defines: its functions and its globals.
fn names_defined_here(unit_program: &RcProgram) -> impl Iterator<Item = &FullName> + '_ {
    unit_program
        .funcs
        .keys()
        .map(|fref| &fref.name)
        .chain(unit_program.globals.iter().map(|global| &global.symbol))
}

/// Whether `unit_program` defines `name` itself.
fn defines(unit_program: &RcProgram, name: &FullName) -> bool {
    names_defined_here(unit_program).any(|defined| defined == name)
}

/// The names the bodies of `unit_program` mention without defining them.
///
/// The names it defines are collected once and looked up by hash, since the walk asks after every
/// mention of every body and the copying below repeats the walk until it finds nothing new.
fn names_reached_elsewhere(unit_program: &RcProgram, mut visit: impl FnMut(&FullName)) {
    let defined: Set<&FullName> = names_defined_here(unit_program).collect();
    let bodies = unit_program
        .funcs
        .values()
        .map(|func| &func.body)
        .chain(unit_program.globals.iter().map(|global| &global.init));
    for body in bodies {
        collect_mentions(body, &mut |mentioned| {
            if !defined.contains(mentioned) {
                visit(mentioned);
            }
        });
    }
}

/// Give each unit its own copy of every function and global accessor it reaches in another, and
/// return what each unit took and which globals ended up shared.
///
/// A copy lets LLVM see the body at the call instead of a call to a symbol it must assume anything
/// may reach. A global's copy is its accessor alone: the initializer stays where the owning unit
/// computes it, and the storage the copy reads is the owner's, so a program reading one global
/// computes it once.
///
/// Copying is a fixed point, since a copied body reaches names of its own.
fn import_what_each_unit_reaches(
    unit_programs: &mut [RcProgram],
    copyable_funcs: &Map<FullName, RcFunc>,
    all_globals: &Map<FullName, RcGlobalInit>,
) -> (Vec<Set<FullName>>, Set<FullName>) {
    let mut imported: Vec<Set<FullName>> = vec![Set::default(); unit_programs.len()];
    let mut shared_globals: Set<FullName> = Set::default();
    loop {
        let mut copied = false;
        for index in 0..unit_programs.len() {
            let mut wanted: Set<FullName> = Set::default();
            names_reached_elsewhere(&unit_programs[index], |mentioned| {
                if copyable_funcs.contains_key(mentioned) || all_globals.contains_key(mentioned) {
                    wanted.insert(mentioned.clone());
                }
            });
            for name in wanted {
                match copyable_funcs.get(&name) {
                    Some(func) => {
                        unit_programs[index]
                            .funcs
                            .insert(FuncRef { name: name.clone() }, func.clone());
                        imported[index].insert(name);
                    }
                    None => take_a_copy_of_the_accessor(
                        &mut unit_programs[index],
                        &mut imported[index],
                        &mut shared_globals,
                        &all_globals[&name],
                    ),
                }
                copied = true;
            }
        }
        if !copied {
            return (imported, shared_globals);
        }
    }
}

/// Give `unit_program` the copy of `global` a unit that does not own it carries: the accessor,
/// reading the owner's storage and calling the owner's initializer. The name is recorded as one the
/// unit took, and as one whose storage more than one unit reaches.
fn take_a_copy_of_the_accessor(
    unit_program: &mut RcProgram,
    imported: &mut Set<FullName>,
    shared_globals: &mut Set<FullName>,
    global: &RcGlobalInit,
) {
    let mut copy = global.clone();
    copy.owns_storage = false;
    unit_program.globals.push(copy);
    imported.insert(global.symbol.clone());
    shared_globals.insert(global.symbol.clone());
}

/// Give the main unit a copy of every value the C world enters the program through.
///
/// It reads them from the entry point and the exported C functions it builds rather than from an RC
/// IR body, so the fixed point above has no mention to find.
fn give_the_main_unit_the_root_values(
    unit_programs: &mut [RcProgram],
    imported: &mut [Set<FullName>],
    shared_globals: &mut Set<FullName>,
    all_globals: &Map<FullName, RcGlobalInit>,
    root_value_names: &Set<FullName>,
) {
    let main_unit = unit_programs.len() - 1;
    for name in root_value_names {
        if defines(&unit_programs[main_unit], name) {
            continue;
        }
        // A root value of funptr type is a function rather than a global, so it has no accessor to
        // copy and the main unit reaches it by declaring it.
        let Some(global) = all_globals.get(name) else {
            continue;
        };
        take_a_copy_of_the_accessor(
            &mut unit_programs[main_unit],
            &mut imported[main_unit],
            shared_globals,
            global,
        );
    }
}

/// The names a unit gives external linkage: the values the C world enters the program through, and
/// every name a unit that has no copy of it reaches.
fn published_to_the_linker(
    unit_programs: &[RcProgram],
    unit_of: &impl Fn(&FullName) -> Option<usize>,
    root_value_names: &Set<FullName>,
) -> Set<FullName> {
    let mut published = root_value_names.clone();
    for unit_program in unit_programs {
        names_reached_elsewhere(unit_program, |mentioned| {
            // A mention no symbol of the program owns is a runtime function or a C declaration,
            // which carries the linkage its own definition gives it.
            if mentioned.is_global() && unit_of(mentioned).is_some() {
                published.insert(mentioned.clone());
            }
        });
    }
    published
}

/// Everything a compilation unit's generated code is decided by, in a form whose bytes are the same
/// exactly when that code is.
///
/// The lists are ordered by name, so that the order the division inserted the functions and globals
/// in — which follows the whole program's, and moves when a symbol elsewhere is added or removed —
/// stays out of the digest.
#[derive(Serialize)]
struct GeneratedCode<'a> {
    /// The settings the code is generated under: the target, the optimization level, the sanitizer.
    settings: &'a str,
    /// The symbols this unit was given, so that two units generating no code are still two units.
    symbols: Vec<String>,
    /// The functions the unit defines, its copies of other units' included.
    funcs: Vec<&'a RcFunc>,
    /// The globals the unit defines, its copies of other units' accessors included.
    globals: Vec<&'a RcGlobalInit>,
    /// The names the unit publishes to the linker, which decides the linkage it defines them under.
    published: Vec<&'a FullName>,
    /// The globals whose storage the unit shares with another, which decides the linkage of the
    /// storage and of the initialization flag.
    shared_globals: Vec<&'a FullName>,
    /// The type each name the unit declares is declared from, by identity.
    declared: Vec<(FullName, u64)>,
    /// The declaration of each type the unit's code is laid out by, by name.
    type_declarations: Vec<(String, u64)>,
    /// Where the unit's code is written, for a build that compiles that into debug information.
    /// `None` for a build that writes none.
    debug_positions: Option<u64>,
    /// What the main unit builds beside its own code: the C entry point and the C function of each
    /// `FFI_EXPORT` statement. `None` for every other unit.
    ///
    /// These are the only parts of a unit's code that the RC IR above does not carry, and the main
    /// unit holds no symbol of the program, so without them two programs would give their main
    /// units one digest and each would link the other's entry point.
    entry: Option<MainUnitEntry>,
}

/// The C functions the main unit builds, as what they are built from.
#[derive(Serialize)]
struct MainUnitEntry {
    /// The `IO ()` action the C entry point runs, as it is written.
    entry_io_value: Option<String>,
    /// The C name and the exported value of each `FFI_EXPORT` statement.
    exports: Vec<(String, String)>,
}

/// The digest naming the object file a compilation unit's code is compiled into.
///
/// The RC IR is built and optimized over the whole program, so a unit's code follows an edit to a
/// module the unit does not depend on: a caller elsewhere asks for a clone of a function this unit
/// holds, the whole-program ownership inference changes what one of its functions borrows, or the
/// division hands it a copy of a body that changed. A digest taken over the sources the unit's
/// symbols are compiled from would not move with any of those, and the build would link an object
/// file for code it no longer generates. This one is taken over the code itself.
pub fn generated_code_hash(
    unit: &CompileUnit,
    unit_index: usize,
    division: &DividedProgram,
    program_for_the_entry: Option<&Program>,
    type_env: &TypeEnv,
    config: &Configuration,
) -> String {
    let unit_program = &division.unit_programs[unit_index];
    let by_name = |a: &&FullName, b: &&FullName| a.cmp(b);
    let mut funcs: Vec<&RcFunc> = unit_program.funcs.values().collect();
    funcs.sort_by(|a, b| a.name.name.cmp(&b.name.name));
    let mut globals: Vec<&RcGlobalInit> = unit_program.globals.iter().collect();
    globals.sort_by(|a, b| a.symbol.cmp(&b.symbol));
    let mut published: Vec<&FullName> = division.published_here[unit_index].iter().collect();
    published.sort_by(by_name);
    let mut shared_globals: Vec<&FullName> = unit_program
        .globals
        .iter()
        .map(|global| &global.symbol)
        .filter(|symbol| division.shared_globals.contains(*symbol))
        .collect();
    shared_globals.sort_by(by_name);

    let mut declared_types: Vec<(FullName, Arc<TypeNode>)> = vec![];
    names_reached_elsewhere(unit_program, |mentioned| {
        // The mentions naming a global of the program are the ones the unit declares; the rest are
        // its local variables and the runtime functions, declared from the runtime's own signature.
        if let Some(ty) = division.global_types.get(mentioned) {
            declared_types.push((mentioned.clone(), ty.clone()));
        }
    });
    declared_types.sort_by(|a, b| a.0.cmp(&b.0));
    declared_types.dedup_by(|a, b| a.0 == b.0);
    let declared: Vec<(FullName, u64)> = declared_types
        .iter()
        .map(|(name, ty)| (name.clone(), ty.type_hash()))
        .collect();
    let type_declarations = type_declarations_reached(unit_program, &declared_types, type_env);
    let debug_positions = config.debug_info.then(|| debug_positions(&funcs, &globals));

    let settings = config.object_generation_hash();
    let code = GeneratedCode {
        settings: &settings,
        symbols: unit
            .symbols()
            .iter()
            .map(|symbol| symbol.name.to_string())
            .collect(),
        funcs,
        globals,
        published,
        shared_globals,
        declared,
        type_declarations,
        debug_positions,
        entry: program_for_the_entry.map(main_unit_entry),
    };
    let mut hash_source = HashSource::default();
    hash_source.push_bytes(
        &postcard::to_allocvec(&code).expect("the generated code of a unit serializes"),
    );
    hash_source.finish()
}

/// The declaration of every type the unit's code is laid out by, each with a digest of what that
/// declaration decides, ordered by the type's name.
///
/// A type reaches the RC IR as a type expression and a field of it as the index of that field, so
/// how wide a value of the type is, where each of its fields sits, and whether it is a pointer are
/// decided by the declarations `type_env` holds rather than by anything the IR carries. Widening a
/// type that a body reads another type's field through moves no part of that body, so the digest
/// reads the declarations beside the IR.
///
/// A declaration lays a value out through the types of its fields, so what those name is read as
/// well, to the end of the chain.
fn type_declarations_reached(
    unit_program: &RcProgram,
    declared: &[(FullName, Arc<TypeNode>)],
    type_env: &TypeEnv,
) -> Vec<(String, u64)> {
    let mut tycons: Set<TyCon> = Set::default();
    for func in unit_program.funcs.values() {
        func.fn_ty.collect_tycons(&mut tycons);
        func.ret_ty.collect_tycons(&mut tycons);
        for param in &func.params {
            param.ty.collect_tycons(&mut tycons);
        }
        if let Some(capture) = &func.capture {
            capture.ty.collect_tycons(&mut tycons);
        }
        for_each_var(&func.body, &mut |var| var.ty.collect_tycons(&mut tycons));
    }
    for global in &unit_program.globals {
        global.ty.collect_tycons(&mut tycons);
        for_each_var(&global.init, &mut |var| var.ty.collect_tycons(&mut tycons));
    }
    for (_, ty) in declared {
        ty.collect_tycons(&mut tycons);
    }

    let mut pending: Vec<TyCon> = tycons.iter().cloned().collect();
    while let Some(tycon) = pending.pop() {
        for field in &declaration_of(&tycon, type_env).fields {
            let mut reached: Set<TyCon> = Set::default();
            field.ty.collect_tycons(&mut reached);
            for reached in reached {
                if tycons.insert(reached.clone()) {
                    pending.push(reached);
                }
            }
        }
    }

    let mut declarations: Vec<(String, u64)> = tycons
        .iter()
        .map(|tycon| {
            (
                tycon.to_string(),
                declaration_digest(declaration_of(tycon, type_env)),
            )
        })
        .collect();
    declarations.sort();
    declarations
}

/// The digest of where the unit's code is written: the source of each function, of each expression
/// node of its body, and of each variable it binds.
///
/// A build that writes debug information compiles these positions into it, so they decide the code
/// it generates. A build that writes none generates the same code wherever the source sits, and a
/// digest reading the positions there would regenerate a unit for an edit that inserts a byte ahead
/// of them — a blank line, a comment, a longer name earlier in the file.
fn debug_positions(funcs: &[&RcFunc], globals: &[&RcGlobalInit]) -> u64 {
    let mut hasher = DefaultHasher::new();
    let mut note = |source: &Option<Span>| {
        source.is_some().hash(&mut hasher);
        if let Some(span) = source {
            span.input.file_path.to_string_lossy().hash(&mut hasher);
            span.start.hash(&mut hasher);
            span.end.hash(&mut hasher);
        }
    };
    for func in funcs {
        note(&func.source);
        for param in &func.params {
            note(&param.source);
        }
        if let Some(capture) = &func.capture {
            note(&capture.source);
        }
        for_each_node(&func.body, &mut |node| note(&node.source));
        for_each_var(&func.body, &mut |var| note(&var.source));
    }
    for global in globals {
        for_each_node(&global.init, &mut |node| note(&node.source));
        for_each_var(&global.init, &mut |var| note(&var.source));
    }
    hasher.finish()
}

/// The declaration `type_env` holds for `tycon`, which every type the generated code is laid out by
/// has.
fn declaration_of<'a>(tycon: &TyCon, type_env: &'a TypeEnv) -> &'a TyConInfo {
    type_env.tycons().get(tycon).unwrap_or_else(|| {
        panic!(
            "the type `{}` the code is generated from is undeclared",
            tycon.to_string()
        )
    })
}

/// The digest of what a declaration decides about the values of the type: how wide one is, where
/// each of its fields sits, and whether it is a pointer. Where the declaration is written and what
/// it documents decide none of that, and stay out.
fn declaration_digest(info: &TyConInfo) -> u64 {
    let mut hasher = DefaultHasher::new();
    info.kind.to_string().hash(&mut hasher);
    info.variant.hash(&mut hasher);
    info.is_unbox.hash(&mut hasher);
    for tyvar in &info.tyvars {
        tyvar.name.hash(&mut hasher);
        tyvar.kind.to_string().hash(&mut hasher);
    }
    for field in &info.fields {
        field.name.hash(&mut hasher);
        field.ty.type_hash().hash(&mut hasher);
        field.is_punched.hash(&mut hasher);
    }
    info.punched_from
        .as_ref()
        .map(|punched_from| punched_from.to_string())
        .hash(&mut hasher);
    hasher.finish()
}

/// What the main unit builds beside its own code, read off the program.
fn main_unit_entry(program: &Program) -> MainUnitEntry {
    let mut exports: Vec<(String, String)> = program
        .export_statements
        .iter()
        .map(|stmt| {
            let value = match &stmt.value_expr {
                Some(expr) => expr.expr.stringify().to_string(),
                None => stmt.value_name.to_string(),
            };
            (stmt.function_name.clone(), value)
        })
        .collect();
    exports.sort();
    MainUnitEntry {
        entry_io_value: program
            .entry_io_value
            .as_ref()
            .map(|expr| expr.expr.stringify().to_string()),
        exports,
    }
}
