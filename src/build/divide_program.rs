//! Dividing the program's RC IR among the compilation units that generate code from it.
//!
//! The RC IR is built and optimized over the whole program, so that a pass sees every call of every
//! function it rewrites. Code generation then runs per unit, so the optimized program is divided
//! here: its entries — the top-level functions and the global values — are dealt out among the
//! units, each unit takes a copy of the small functions and the accessors it reaches in another, a
//! global one unit reads is given to that unit, and a name is published to the linker only where a
//! unit that has no copy of it names it.

use crate::ast::name::FullName;
use crate::ast::program::{Program, TypeEnv};
use crate::ast::types::{TyCon, TyConInfo, TypeNode};
use crate::build::compile_unit::CompileUnit;
use crate::configuration::Configuration;
use crate::constants::WHOLE_PROGRAM_IN_ONE_UNIT;
use crate::hash::HashSource;
use crate::misc::{split_at_name_boundaries, Map, Set};
use crate::parse::sourcefile::Span;
use crate::rc_ir::ast::{
    for_each_node, for_each_var, FuncRef, RcExprNode, RcFunc, RcGlobalInit, RcProgram,
};
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

/// The most RC IR nodes a global's initializer may add to the unit that alone reads the value for
/// the initializer to travel there with the storage. A unit generating the initializer of the value
/// it reads optimizes the reads by what the initializer settles — the length of an array, the shape
/// of a structure — which is what takes the bounds checks out of a loop reading the global. What it
/// costs is the initializer and the bodies it reaches that the unit does not already hold, in the
/// unit a program's own edits regenerate, so an initializer that would bring a graph of them along
/// stays where it is.
const MOVED_INITIALIZER_NODE_LIMIT: u64 = 200;

/// The program's RC IR divided among the compilation units, and what a unit needs to know about the
/// others to generate code from its own slice.
// PROOF: P27, P29, P30 (dev-docs/proof/rc_ir/borrow-cancel)
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
    /// The globals whose storage more than one unit reaches. The unit keeping the value defines
    /// and publishes the storage, and the units holding a copy of the accessor declare it, so one
    /// storage holds the value however many units read it. A global one unit reads is kept by that
    /// unit and is not here, so nothing about it is published.
    pub shared_globals: Arc<Set<FullName>>,
}

/// Divide `program` — the whole program's RC IR, optimized — among `units`, which
/// `divide_into_units` made out of the entries of that same program.
///
/// # Arguments
/// * `global_types` — the type of every symbol of the program, which the types of the versions the
///   optimizer synthesized are added to.
/// * `root_value_names` — the values the C world enters the program through.
// PROOF: D/A, P27, P29, P30 (dev-docs/proof/rc_ir/borrow-cancel)
pub fn divide_among_units(
    program: RcProgram,
    units: &[CompileUnit],
    global_types: &Map<FullName, Arc<TypeNode>>,
    root_value_names: Set<FullName>,
) -> DividedProgram {
    // The main unit is the last, and it is the one holding no entry of the program: it builds the
    // entry point and the exported C functions, which is where the root values are read.
    let main_unit_entries = units
        .last()
        .expect("a program is divided into at least the main unit")
        .entries();
    assert!(
        main_unit_entries.is_empty(),
        "the last compilation unit holds {} entries, and the main unit holds none",
        main_unit_entries.len()
    );
    let unit_of = unit_of_each_entry(units);
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
        let index = unit_of[&fref.name];
        unit_programs[index].funcs.insert(fref, func);
    }
    for global in program.globals {
        let index = unit_of[&global.symbol];
        unit_programs[index].globals.push(global);
    }

    let mut imported: Vec<Set<FullName>> = vec![Set::default(); units.len()];
    let mut shared_globals: Set<FullName> = Set::default();
    import_what_each_unit_reaches(
        &mut unit_programs,
        &mut imported,
        &mut shared_globals,
        &copyable_funcs,
        &all_globals,
    );
    give_the_main_unit_the_root_values(
        &mut unit_programs,
        &mut imported,
        &mut shared_globals,
        &all_globals,
        &root_value_names,
    );

    // A unit reads a global through the code it generates, so which units read one is answered
    // once the bodies no unit reaches are gone.
    let (mut published, mut published_here) = publish_and_prune(
        &mut unit_programs,
        &unit_of,
        &imported,
        &shared_globals,
        &root_value_names,
    );
    if move_each_initializer_to_the_unit_that_alone_reads_it(
        &mut unit_programs,
        &shared_globals,
        &root_value_names,
        &copyable_funcs,
    ) {
        // The unit an initializer moved to generates a body it used to declare, and the pruning
        // above dropped what that body reaches: a unit carrying the accessor alone generates no
        // initializer, so nothing there reached those bodies. The unit it moved out of is left
        // holding whatever only that initializer reached, which the pruning then drops, so that
        // the reads each unit performs are the ones its own code performs.
        import_what_each_unit_reaches(
            &mut unit_programs,
            &mut imported,
            &mut shared_globals,
            &copyable_funcs,
            &all_globals,
        );
        (published, published_here) = publish_and_prune(
            &mut unit_programs,
            &unit_of,
            &imported,
            &shared_globals,
            &root_value_names,
        );
    }
    if keep_each_global_where_it_is_read(&mut unit_programs, &mut shared_globals, &root_value_names)
    {
        (published, published_here) = publish_and_prune(
            &mut unit_programs,
            &unit_of,
            &imported,
            &shared_globals,
            &root_value_names,
        );
    }
    assert_each_unit_serves_the_globals_it_reads(&unit_programs, &shared_globals, &all_globals);

    DividedProgram {
        unit_programs,
        published: Arc::new(published),
        global_types: Arc::new(global_types),
        imported,
        published_here,
        shared_globals: Arc::new(shared_globals),
    }
}

/// Divide the entries of `program` — its top-level functions and its global values — into
/// compilation units averaging `config.cu_size` entries each, and add the main unit.
///
/// Where a unit ends is decided by the names of the entries it holds, so an entry the program gains
/// moves the boundaries of at most two units and the rest keep their cached object files.
///
/// `FullName` orders by the rendered name, so the functions lifted or cloned out of one symbol,
/// whose names extend that symbol's, sit beside it and land in the same unit as often as a boundary
/// allows.
///
/// The main unit comes last and holds no entry: it builds the C entry point and the exported C
/// functions, which are the code of nothing the program defines.
///
/// A `cu_size` of `WHOLE_PROGRAM_IN_ONE_UNIT` puts every entry in one unit, so that LLVM sees every
/// call the program makes. The band a boundary falls in is one `cu_size` wide, so a size that large
/// would leave one unit for all but two of the hashes a name can take; this answers for every name
/// instead.
// PROOF: D/A, P27, P29, P30 (dev-docs/proof/rc_ir/borrow-cancel)
pub fn divide_into_units(program: &RcProgram, config: &Configuration) -> Vec<CompileUnit> {
    let mut entries: Vec<FullName> = program
        .funcs
        .keys()
        .map(|fref| fref.name.clone())
        .chain(program.globals.iter().map(|global| global.symbol.clone()))
        .collect();
    entries.sort();
    // Each entry belongs to one unit, and `unit_of_each_entry` reads the division back by name, so
    // one name standing for two entries would put one of them in the other's unit.
    if let Some(pair) = entries.windows(2).find(|pair| pair[0] == pair[1]) {
        panic!(
            "the program defines `{}` twice, so no unit is the one it belongs to",
            pair[0].to_string()
        );
    }
    let pieces = if config.cu_size == WHOLE_PROGRAM_IN_ONE_UNIT {
        vec![entries]
    } else {
        split_at_name_boundaries(entries, config.cu_size, FullName::to_string)
    };
    let mut units: Vec<CompileUnit> = pieces.into_iter().map(CompileUnit::new).collect();
    units.push(CompileUnit::new(vec![]));
    units
}

/// Which unit each of the program's entries belongs to, by name.
///
/// The keys are every name the program defines, so a mention this does not answer for is a runtime
/// function or a C declaration, which carries the linkage its own definition gives it.
fn unit_of_each_entry(units: &[CompileUnit]) -> Map<FullName, usize> {
    let mut unit_of = Map::default();
    for (index, unit) in units.iter().enumerate() {
        for entry in unit.entries() {
            unit_of.insert(entry.clone(), index);
        }
    }
    unit_of
}

/// The type of every global the program defines, the versions the optimizer synthesized included.
///
/// A unit declares a name another unit defines from this, and a synthesized version is not among
/// the program's symbols, so its own type is all there is to declare it from.
// PROOF: P27, P29, P30 (dev-docs/proof/rc_ir/borrow-cancel)
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
// PROOF: P27, P29, P30 (dev-docs/proof/rc_ir/borrow-cancel)
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

/// The bodies `unit_program` generates code from: its functions, and the initializers of the
/// globals it computes. The initializer of a global another unit computes is carried here to say
/// what the value is, and this unit generates none of it.
fn bodies_generated_here(unit_program: &RcProgram) -> impl Iterator<Item = &RcExprNode> + '_ {
    unit_program.funcs.values().map(|func| &func.body).chain(
        unit_program
            .globals
            .iter()
            .filter(|global| global.owns_initializer)
            .map(|global| &global.init),
    )
}

/// The names the bodies `unit_program` generates mention without defining them, which are the names
/// its code declares.
///
/// The names it defines are collected once and looked up by hash, since the walk asks after every
/// mention of every body and the copying below repeats the walk until it finds nothing new.
fn names_reached_elsewhere(unit_program: &RcProgram, mut visit: impl FnMut(&FullName)) {
    let defined: Set<&FullName> = names_defined_here(unit_program).collect();
    for body in bodies_generated_here(unit_program) {
        collect_mentions(body, &mut |mentioned| {
            if !defined.contains(mentioned) {
                visit(mentioned);
            }
        });
    }
}

/// Give each unit its own copy of every function and global accessor it reaches in another,
/// recording in `imported` what each unit took and in `shared_globals` which globals ended up
/// shared.
///
/// A copy lets LLVM see the body at the call instead of a call to a symbol it must assume anything
/// may reach. A global's copy is its accessor alone: the initializer stays where the owning unit
/// computes it, and the storage the copy reads is the owner's, so a program reading one global
/// computes it once.
///
/// Copying is a fixed point, since a copied body reaches names of its own.
// PROOF: D/A, P27, P29, P30 (dev-docs/proof/rc_ir/borrow-cancel)
fn import_what_each_unit_reaches(
    unit_programs: &mut [RcProgram],
    imported: &mut [Set<FullName>],
    shared_globals: &mut Set<FullName>,
    copyable_funcs: &Map<FullName, RcFunc>,
    all_globals: &Map<FullName, RcGlobalInit>,
) {
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
                        shared_globals,
                        &all_globals[&name],
                    ),
                }
                copied = true;
            }
        }
        if !copied {
            return;
        }
    }
}

/// Move the initializer of each global one unit alone reads into that unit, where what the move
/// brings along fits there, and report whether any moved.
///
/// A unit generating the initializer of the value it reads optimizes the reads by what the
/// initializer settles — the length of an array, the shape of a structure — which is what takes the
/// bounds checks out of a loop reading the global. One that would bring a graph of bodies along
/// stays where it is, since that graph would land in the unit a program's own edits regenerate;
/// `MOVED_INITIALIZER_NODE_LIMIT` says how much is little.
///
/// The storage follows afterwards, in `keep_each_global_where_it_is_read`: an initializer that
/// moves takes the reads it performs to the unit it moves to, so which units read which global is
/// settled once the initializers are where they belong.
///
/// A root value is read by the entry point and by the exported C functions, which the main unit
/// builds after the division out of no body this walk can read, so its initializer stays where the
/// division put it.
fn move_each_initializer_to_the_unit_that_alone_reads_it(
    unit_programs: &mut [RcProgram],
    shared_globals: &Set<FullName>,
    root_value_names: &Set<FullName>,
    copyable_funcs: &Map<FullName, RcFunc>,
) -> bool {
    let readers = units_reading_each_global(unit_programs, shared_globals);
    // The unit each initializer that moves is moving to.
    let mut destinations: Map<FullName, usize> = Map::default();
    for (name, readers) in &readers {
        if root_value_names.contains(name) || readers.len() != 1 {
            continue;
        }
        let reader = *readers.iter().next().unwrap();
        if unit_computing(unit_programs, name) == reader
            || !initializer_fits(unit_programs, name, reader, copyable_funcs)
        {
            continue;
        }
        destinations.insert(name.clone(), reader);
    }
    for (index, unit_program) in unit_programs.iter_mut().enumerate() {
        for global in &mut unit_program.globals {
            if let Some(destination) = destinations.get(&global.symbol) {
                global.owns_initializer = *destination == index;
            }
        }
    }
    !destinations.is_empty()
}

/// The unit that computes the value of the global `name`.
fn unit_computing(unit_programs: &[RcProgram], name: &FullName) -> usize {
    unit_programs
        .iter()
        .position(|unit_program| {
            unit_program
                .globals
                .iter()
                .any(|global| global.symbol == *name && global.owns_initializer)
        })
        .unwrap_or_else(|| panic!("no unit computes the value of `{}`", name.to_string()))
}

/// Keep each global in the units that read it, and report whether any of them changed hands.
///
/// The unit alone reading a global keeps the value, where nothing about it is published: a unit
/// reading a global another unit keeps reads storage the linker publishes, and LLVM has to assume
/// that a store anywhere in the unit writes it — the test of the initialization flag and the load
/// of the storage stay inside every loop that reads the global, and so do the bounds checks that
/// the lifted load would have taken out. A value more than one unit reads stays where it is and is
/// published from there, so that one storage serves every reader.
///
/// Which units read a global is read off the bodies each unit generates, which is why this comes
/// after the initializers have moved and the copying and the pruning have caught up with them:
/// moving an initializer moves the reads it performs, and a unit generating a body that reads a
/// global has to be one that can serve the read.
///
/// A global no unit reads computes a value nothing observes, and no unit is left holding a part of
/// it. A root value is read by the entry point and by the exported C functions, which the main unit
/// builds after the division out of no body this walk can read, so it is left as the division put
/// it: kept and published by the unit defining it, and read through the copy the main unit carries.
fn keep_each_global_where_it_is_read(
    unit_programs: &mut [RcProgram],
    shared_globals: &mut Set<FullName>,
    root_value_names: &Set<FullName>,
) -> bool {
    let placed: Set<FullName> = unit_programs
        .iter()
        .flat_map(|unit_program| unit_program.globals.iter().map(|global| &global.symbol))
        .filter(|name| !root_value_names.contains(*name))
        .cloned()
        .collect();
    let readers = units_reading_each_global(unit_programs, &placed);
    let keepers: Map<&FullName, Option<usize>> = readers
        .iter()
        .map(|(name, readers)| (name, keeper_of(unit_programs, name, readers)))
        .collect();

    let mut changed = false;
    for (index, unit_program) in unit_programs.iter_mut().enumerate() {
        unit_program.globals.retain_mut(|global| {
            let Some(readers) = readers.get(&global.symbol) else {
                return true;
            };
            let keeps = keepers[&global.symbol] == Some(index);
            changed |= global.owns_storage != keeps;
            global.owns_storage = keeps;
            // A value nothing reads is computed by nobody.
            if keepers[&global.symbol].is_none() {
                changed |= global.owns_initializer;
                global.owns_initializer = false;
            }
            // A unit that neither keeps the value nor computes it nor reads it holds no part of the
            // global.
            let held = keeps || global.owns_initializer || readers.contains(&index);
            changed |= !held;
            held
        });
    }
    for (name, readers) in &readers {
        changed |= if readers.len() > 1 {
            shared_globals.insert(name.clone())
        } else {
            shared_globals.remove(name)
        };
    }
    changed
}

/// The unit that keeps the value of the global `name`, given the units reading it: the one unit
/// reading it, or, where more than one does, the unit already keeping it. A value nothing reads is
/// kept by nobody.
fn keeper_of(unit_programs: &[RcProgram], name: &FullName, readers: &Set<usize>) -> Option<usize> {
    let mut readers: Vec<usize> = readers.iter().copied().collect();
    readers.sort();
    match readers.as_slice() {
        [] => None,
        [reader] => Some(*reader),
        [first, ..] => Some(
            unit_programs
                .iter()
                .position(|unit_program| {
                    unit_program
                        .globals
                        .iter()
                        .any(|global| global.symbol == *name && global.owns_storage)
                })
                .unwrap_or(*first),
        ),
    }
}

/// Check that every unit can serve each read of a global its own code performs, and that one unit
/// computes each global's value and one keeps it.
///
/// A unit reads a global through an accessor of its own, which tests the initialization flag and
/// loads the storage: the unit keeps the value, or it reads storage the unit keeping it publishes.
/// A unit that does neither declares an accessor with internal linkage and no body, which the LLVM
/// verifier rejects at the end of a build that has already done all of its work.
fn assert_each_unit_serves_the_globals_it_reads(
    unit_programs: &[RcProgram],
    shared_globals: &Set<FullName>,
    all_globals: &Map<FullName, RcGlobalInit>,
) {
    for (index, unit_program) in unit_programs.iter().enumerate() {
        let served: Set<&FullName> = unit_program
            .globals
            .iter()
            .filter(|global| global.owns_storage || shared_globals.contains(&global.symbol))
            .map(|global| &global.symbol)
            .collect();
        for body in bodies_generated_here(unit_program) {
            collect_mentions(body, &mut |mentioned| {
                assert!(
                    !all_globals.contains_key(mentioned) || served.contains(mentioned),
                    "unit {} generates a body reading `{}`, and neither keeps that value nor \
                     reads the storage of a unit that does",
                    index,
                    mentioned.to_string()
                );
            });
        }
    }
}

/// Whether moving the initializer of the global `name` into unit `reader` adds no more than
/// `MOVED_INITIALIZER_NODE_LIMIT` nodes to that unit.
///
/// What it adds is the initializer and the bodies it reaches that the unit does not already hold,
/// which is what the copying would give it once the unit generates the initializer. The walk stops
/// as soon as the total is past the limit, so a large graph costs a small walk.
fn initializer_fits(
    unit_programs: &[RcProgram],
    name: &FullName,
    reader: usize,
    copyable_funcs: &Map<FullName, RcFunc>,
) -> bool {
    let global = unit_programs
        .iter()
        .flat_map(|unit_program| unit_program.globals.iter())
        .find(|global| global.symbol == *name && global.owns_initializer)
        .unwrap_or_else(|| panic!("no unit computes the value of `{}`", name.to_string()));
    let held: Set<&FullName> = names_defined_here(&unit_programs[reader]).collect();
    let mut nodes = node_count(&global.init);
    let mut walked: Set<FullName> = Set::default();
    let mut pending: Vec<&RcExprNode> = vec![&global.init];
    while let Some(body) = pending.pop() {
        if nodes > MOVED_INITIALIZER_NODE_LIMIT {
            return false;
        }
        let mut reached: Vec<FullName> = vec![];
        collect_mentions(body, &mut |mentioned| {
            if !held.contains(mentioned) && !walked.contains(mentioned) {
                reached.push(mentioned.clone());
            }
        });
        for mentioned in reached {
            let Some(func) = copyable_funcs.get(&mentioned) else {
                continue;
            };
            if !walked.insert(mentioned) {
                continue;
            }
            nodes += node_count(&func.body);
            pending.push(&func.body);
        }
    }
    nodes <= MOVED_INITIALIZER_NODE_LIMIT
}

/// Which units read each of `globals`, by name.
///
/// A unit reads a global when a body it generates mentions it: one of its functions, or the
/// initializer of a global it computes.
fn units_reading_each_global(
    unit_programs: &[RcProgram],
    globals: &Set<FullName>,
) -> Map<FullName, Set<usize>> {
    let mut readers: Map<FullName, Set<usize>> = globals
        .iter()
        .map(|name| (name.clone(), Set::default()))
        .collect();
    for (index, unit_program) in unit_programs.iter().enumerate() {
        for body in bodies_generated_here(unit_program) {
            collect_mentions(body, &mut |mentioned| {
                if let Some(units) = readers.get_mut(mentioned) {
                    units.insert(index);
                }
            });
        }
    }
    readers
}

/// Give `unit_program` the copy of `global` a unit that computes neither its value nor its storage
/// carries: the accessor, reading the storage and calling the initializer another unit publishes.
/// The name is recorded as one the unit took, and as one whose storage more than one unit reaches.
fn take_a_copy_of_the_accessor(
    unit_program: &mut RcProgram,
    imported: &mut Set<FullName>,
    shared_globals: &mut Set<FullName>,
    global: &RcGlobalInit,
) {
    let mut copy = global.clone();
    copy.owns_initializer = false;
    copy.owns_storage = false;
    unit_program.globals.push(copy);
    imported.insert(global.symbol.clone());
    shared_globals.insert(global.symbol.clone());
}

/// Give the main unit a copy of every value the C world enters the program through.
///
/// It reads them from the entry point and the exported C functions it builds rather than from an RC
/// IR body, so `import_what_each_unit_reaches` has no mention to find.
fn give_the_main_unit_the_root_values(
    unit_programs: &mut [RcProgram],
    imported: &mut [Set<FullName>],
    shared_globals: &mut Set<FullName>,
    all_globals: &Map<FullName, RcGlobalInit>,
    root_value_names: &Set<FullName>,
) {
    let main_unit_index = unit_programs.len() - 1;
    for name in root_value_names {
        if defines(&unit_programs[main_unit_index], name) {
            continue;
        }
        // A root value of funptr type is a function rather than a global, so it has no accessor to
        // copy and the main unit reaches it by declaring it.
        let Some(global) = all_globals.get(name) else {
            continue;
        };
        take_a_copy_of_the_accessor(
            &mut unit_programs[main_unit_index],
            &mut imported[main_unit_index],
            shared_globals,
            global,
        );
    }
}

/// Publish to the linker the names one unit's code reaches in another, drop from each unit what its
/// own code cannot reach, and return both sets of published names: the program's, and one per unit
/// of the names that unit defines under external linkage.
// PROOF: P27, P29, P30 (dev-docs/proof/rc_ir/borrow-cancel)
fn publish_and_prune(
    unit_programs: &mut [RcProgram],
    defined_in_program: &Map<FullName, usize>,
    imported: &[Set<FullName>],
    shared_globals: &Set<FullName>,
    root_value_names: &Set<FullName>,
) -> (Set<FullName>, Vec<Set<FullName>>) {
    let published =
        names_published_to_the_linker(unit_programs, defined_in_program, root_value_names);
    let mut published_here: Vec<Set<FullName>> = vec![];
    for (index, unit_program) in unit_programs.iter_mut().enumerate() {
        published_here.push(
            names_defined_here(unit_program)
                .filter(|name| !imported[index].contains(name) && published.contains(name))
                .cloned()
                .collect(),
        );
        // The initializer of a value another unit keeps is called from that unit, so the unit
        // computing it generates it however little of its own code reads the value.
        let initializers_another_unit_calls: Set<FullName> = unit_program
            .globals
            .iter()
            .filter(|global| global.owns_initializer && !global.owns_storage)
            .map(|global| global.symbol.clone())
            .collect();
        // What the unit's code is reached by: the names another unit's code calls it through, the
        // values the C world enters the program through, which the entry point and the exported C
        // functions read through the accessor the unit holding them carries, the globals whose
        // storage another unit reads, whose keeper emits that storage, and the initializers
        // another unit calls.
        unit_program.roots = names_defined_here(unit_program)
            .filter(|name| {
                published_here[index].contains(*name)
                    || root_value_names.contains(*name)
                    || shared_globals.contains(*name)
                    || initializers_another_unit_calls.contains(*name)
            })
            .cloned()
            .collect();
        // A function every unit calling it took a copy of is left with no caller of its own, and
        // generating it writes out a body that reaches nothing the program runs.
        eliminate_unreachable(unit_program);
    }
    (published, published_here)
}

/// The names a unit gives external linkage: the values the C world enters the program through, and
/// every name a unit that has no copy of it reaches.
fn names_published_to_the_linker(
    unit_programs: &[RcProgram],
    defined_in_program: &Map<FullName, usize>,
    root_value_names: &Set<FullName>,
) -> Set<FullName> {
    let mut published = root_value_names.clone();
    for unit_program in unit_programs {
        names_reached_elsewhere(unit_program, |mentioned| {
            // A mention the program defines no entry for is a runtime function or a C
            // declaration, which carries the linkage its own definition gives it.
            if mentioned.is_global() && defined_in_program.contains_key(mentioned) {
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
    /// The entries this unit was given, so that two units generating no code are still two units.
    entries: Vec<String>,
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
    /// These are the only parts of a unit's code that `funcs` and `globals` do not carry, and the
    /// main unit holds no symbol of the program, so without them two programs would give their
    /// main units one digest and each would link the other's entry point.
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
/// entries are compiled from would not move with any of those, and the build would link an object
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
        entries: unit.entries().iter().map(FullName::to_string).collect(),
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
            let mut field_tycons: Set<TyCon> = Set::default();
            field.ty.collect_tycons(&mut field_tycons);
            for reached in field_tycons {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rc_ir::test_program::{func, global, global_name, prog};

    /// A global whose initializer holds more nodes than one travelling to its reader may bring
    /// along, so that the division leaves it in the unit it was dealt to.
    fn global_too_large_to_move(symbol: FullName, mentions: &[FullName]) -> RcGlobalInit {
        let repeated: Vec<FullName> = mentions
            .iter()
            .cycle()
            .take(MOVED_INITIALIZER_NODE_LIMIT as usize + 1)
            .cloned()
            .collect();
        global(symbol, &repeated)
    }

    /// The type of every symbol of `program`, which a unit declares another unit's name from.
    fn global_types(program: &RcProgram) -> Map<FullName, Arc<TypeNode>> {
        program
            .globals
            .iter()
            .map(|global| (global.symbol.clone(), global.ty.clone()))
            .chain(
                program
                    .funcs
                    .values()
                    .map(|func| (func.name.name.clone(), func.fn_ty.clone())),
            )
            .collect()
    }

    /// The units keeping the value of the global `name`.
    fn units_keeping(division: &DividedProgram, name: &FullName) -> Vec<usize> {
        division
            .unit_programs
            .iter()
            .enumerate()
            .filter(|(_, unit_program)| {
                unit_program
                    .globals
                    .iter()
                    .any(|global| global.symbol == *name && global.owns_storage)
            })
            .map(|(index, _)| index)
            .collect()
    }

    /// The units generating a body that reads the global `name`.
    fn units_reading(division: &DividedProgram, name: &FullName) -> Vec<usize> {
        division
            .unit_programs
            .iter()
            .enumerate()
            .filter(|(_, unit_program)| {
                let mut reads = false;
                for body in bodies_generated_here(unit_program) {
                    collect_mentions(body, &mut |mentioned| reads |= mentioned == name);
                }
                reads
            })
            .map(|(index, _)| index)
            .collect()
    }

    /// The storage of a global goes to the unit that reads it once the initializers have moved.
    ///
    /// An initializer travelling to the unit that alone reads the value takes the reads it performs
    /// with it, so the unit it moves into reads whatever that initializer reads. The value it reads
    /// has to be one that unit keeps or one whose storage a keeper publishes; a unit computing a
    /// value it cannot read declares an accessor with internal linkage and no body, which the LLVM
    /// verifier rejects.
    #[test]
    fn test_the_storage_of_a_global_follows_the_reads_an_initializer_takes_with_it() {
        let (table, holder, entry, source) = (
            global_name("table"),
            global_name("holder"),
            global_name("entry"),
            global_name("source"),
        );
        // `holder` is read by `entry` alone, so its initializer travels to the unit holding
        // `entry`; that initializer reads `table`, whose initializer is too large to travel and
        // stays in that same unit.
        let program = prog(
            vec![
                func(entry.clone(), &[holder.clone()]),
                func(source.clone(), &[]),
            ],
            vec![
                global(holder.clone(), &[table.clone()]),
                global_too_large_to_move(table.clone(), &[source.clone()]),
            ],
            &[],
        );
        let units = vec![
            CompileUnit::new(vec![holder.clone()]),
            CompileUnit::new(vec![entry.clone(), source.clone(), table.clone()]),
            CompileUnit::new(vec![]),
        ];
        let global_types = global_types(&program);

        let division = divide_among_units(
            program,
            &units,
            &global_types,
            [entry.clone()].into_iter().collect(),
        );

        let readers = units_reading(&division, &table);
        let keepers = units_keeping(&division, &table);
        assert_eq!(
            keepers.len(),
            1,
            "one unit should keep the value of `table`, and {:?} do",
            keepers
        );
        assert!(
            !readers.is_empty(),
            "the program should read `table`, and no unit generates a body that does"
        );
        for reader in &readers {
            assert!(
                keepers.contains(reader) || division.shared_globals.contains(&table),
                "unit {} generates a body reading `table`, and neither keeps that value nor \
                 reads the storage of the unit that does",
                reader
            );
        }
        assert!(
            !division.shared_globals.contains(&table),
            "one unit reads `table`, so nothing about it is published"
        );
    }
}
