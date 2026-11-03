// Lambda set specialization: which lambdas a value of function type can be, carried in its type.
//
// Every slot of a type where a function value sits (an arrow, and the arrows a struct, a union or an
// array declares) carries a lambda set variable. Inference unifies the variables of two places whose
// values flow into each other, as the type checker unifies their types, and records each lambda in
// the variable of the place it is built at. A component of the call graph is inferred together and
// generalized over the variables of its symbols' types, so that a function taking a function is
// inferred once and copied for each lambda set it is given. Specialization then walks from the
// program's entry, and gives each copy the concrete lambda sets of its parameters.
//
// The design is laid out in `dev-docs/2026-10-24-lambda-set-specialization/design.md`.

mod concrete;
mod dump;
mod flow;
mod infer;
mod slots;
mod solver;
mod specialize;

use crate::{
    ast::program::Program,
    constants::DOT_FIXLANG,
    error::panic_if_err,
    project_fs::{write_file_guarding_project_links, DOT_FIXLANG_LINK_HINT},
};
pub use flow::BuiltinFlow;
use infer::infer;
use slots::SlotLayout;
use specialize::specialize;
use std::{fs, path::PathBuf};

/// Infers the lambda sets of `prg` and the copies they ask for, leaving the program as it is. With
/// `dump_step`, writes them to a file under `.fixlang` named after it; a failure to write it ends
/// the process with the error printed.
pub fn run(prg: &Program, dump_step: Option<String>) {
    let layout = SlotLayout::new(&prg.type_env);
    let mut analysis = infer(prg, &layout);
    let spec = specialize(&mut analysis);
    if let Some(step) = dump_step {
        let text = dump::render(&mut analysis, &spec, &layout);
        let path = PathBuf::from(format!("{}/{}.lambda_sets.txt", DOT_FIXLANG, step));
        panic_if_err(write_file_guarding_project_links(
            &path,
            DOT_FIXLANG_LINK_HINT,
            |tmp_path| fs::write(tmp_path, &text),
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::{
        concrete::{Elem, SetId},
        infer::{infer, Analysis},
        slots::{AType, SlotLayout},
        specialize::{specialize, Specialization},
    };
    use crate::{
        ast::program::Program,
        configuration::{Configuration, FixOptimizationLevel, SubCommand},
        elaboration::elaborate_via_config,
        error::panic_if_err,
        misc::save_temporary_source,
        optimization::unwrap_newtype,
    };

    /// `source` elaborated into the form the inference reads: monomorphic, curried, and with its
    /// newtypes unwrapped.
    fn program_of(source: &str) -> Program {
        let mut config = panic_if_err(Configuration::release_mode(SubCommand::Build));
        config.set_fix_opt_level(FixOptimizationLevel::Max);
        config.num_worker_thread = 0;
        let saved = panic_if_err(save_temporary_source(source, "lambda_set"));
        config.add_user_source_file(saved.file_path);
        let mut prg = panic_if_err(elaborate_via_config(&config));
        unwrap_newtype::run(&mut prg, 0);
        prg
    }

    /// `source` in the form the pass reads in the compiler: `program_of` after `inline` and
    /// `inline_local`, which leave a getter's body, a builtin reading the struct by name, in its
    /// caller.
    fn program_after_inlining(source: &str) -> Program {
        let mut prg = program_of(source);
        crate::optimization::inline::run(&mut prg, false, 0);
        crate::optimization::inline_local::run(&mut prg, 0);
        prg
    }

    /// The lambda sets of a program and the copies they ask for.
    struct Inferred<'p> {
        prg: &'p Program,
        layout: SlotLayout<'p>,
        analysis: Analysis,
        spec: Specialization,
    }

    impl<'p> Inferred<'p> {
        /// Infers the lambda sets of `prg` and the copies they ask for.
        fn new(prg: &'p Program) -> Self {
            let layout = SlotLayout::new(&prg.type_env);
            let mut analysis = infer(prg, &layout);
            let spec = specialize(&mut analysis);
            Inferred {
                prg,
                layout,
                analysis,
                spec,
            }
        }

        /// The component of the one symbol of `Main` named `name`, and its position there.
        fn component_of(&self, name: &str) -> (usize, usize) {
            let found: Vec<_> = self
                .prg
                .symbols
                .keys()
                .filter(|s| {
                    s.namespace.to_string() == "Main" && s.name.split('#').next() == Some(name)
                })
                .collect();
            assert_eq!(
                found.len(),
                1,
                "the symbols named `{}`: {:?}",
                name,
                found.iter().map(|s| s.to_string()).collect::<Vec<_>>()
            );
            let c = self.analysis.component_of[found[0]];
            let pos = self.analysis.components[c]
                .symbols
                .iter()
                .position(|s| s == found[0])
                .unwrap();
            (c, pos)
        }

        /// The annotated type of the symbol `name` of `Main`.
        fn type_of(&self, name: &str) -> AType {
            let (c, pos) = self.component_of(name);
            self.analysis.components[c].types[pos].clone()
        }

        /// For each copy of the component of `name`, the set it gives the own slot of the type
        /// that `path` reaches from the symbol's type, each step a type argument.
        fn sets_at(&mut self, name: &str, path: &[usize]) -> Vec<SetId> {
            let (c, _) = self.component_of(name);
            let mut ty = self.type_of(name);
            for i in path {
                ty = self.layout.type_arg(&ty, *i);
            }
            let slot = self.layout.own_slots(&ty)[0];
            let root = self.analysis.solver.find(slot);
            let p = self.analysis.components[c]
                .params
                .iter()
                .position(|v| *v == root)
                .unwrap();
            self.spec
                .instances
                .iter()
                .filter(|inst| inst.component == c)
                .map(|inst| inst.args[p])
                .collect()
        }

        /// The elements of `set`, each lambda as the name of the symbol it is written in, and
        /// Dynamic as `None`.
        fn writers(&self, set: SetId) -> Vec<Option<String>> {
            self.spec
                .store
                .elems(set)
                .iter()
                .map(|e| match e {
                    Elem::Dynamic => None,
                    Elem::Lam(lam, _) => Some(
                        self.analysis.lambdas[*lam as usize]
                            .symbol
                            .name
                            .split('#')
                            .next()
                            .unwrap()
                            .to_string(),
                    ),
                })
                .collect()
        }
    }

    /// `v` in ascending order, Dynamic (`None`) first.
    fn sorted(mut v: Vec<Option<String>>) -> Vec<Option<String>> {
        v.sort();
        v
    }

    /// What `Inferred::writers` gives a lambda written in `main`.
    fn main_lam() -> Option<String> {
        Some("main".to_string())
    }

    /// A function given a different lambda at each of two calls gets a copy for each, and in each
    /// the function parameter's set holds the one lambda that call passes.
    #[test]
    fn test_a_function_given_two_lambdas_is_copied_once_for_each() {
        let prg = program_of(
            r#"
module Main;

apply : (I64 -> I64) -> I64 -> I64;
apply = |f, x| f(x);

main : IO ();
main = (
    let n = 10;
    let a = apply(|x| x + n, 1);
    let b = apply(|x| x * 2, 2);
    println((a + b).to_string)
);
"#,
        );
        let mut inf = Inferred::new(&prg);
        let sets = inf.sets_at("apply", &[0]);
        assert_eq!(sets.len(), 2);
        assert_ne!(sets[0], sets[1]);
        for s in &sets {
            assert_eq!(inf.writers(*s), vec![main_lam()]);
        }
    }

    /// A recursion that passes itself a lambda wrapping the function it was given puts that lambda
    /// in the same set as the one from outside: the program asks for one copy, and the set of that
    /// copy holds the wrapping lambda, which captures a function of the set itself.
    #[test]
    fn test_a_recursion_wrapping_its_function_asks_for_one_copy_of_a_set_holding_itself() {
        let prg = program_of(
            r#"
module Main;

rec : (I64 -> I64) -> I64 -> I64;
rec = |f, n| if n == 0 { f(0) } else { rec(|x| f(x) * 2, n - 1) };

main : IO ();
main = (
    let k = 10;
    println(rec(|x| x + k, 3).to_string)
);
"#,
        );
        let mut inf = Inferred::new(&prg);
        let sets = inf.sets_at("rec", &[0]);
        assert_eq!(sets.len(), 1);
        let s = sets[0];
        assert_eq!(
            sorted(inf.writers(s)),
            vec![main_lam(), Some("rec".to_string())]
        );
        let holds_itself = inf.spec.store.elems(s).iter().any(|e| match e {
            Elem::Lam(lam, args) => {
                inf.analysis.lambdas[*lam as usize]
                    .symbol
                    .name
                    .starts_with("rec")
                    && args.contains(&s)
            }
            Elem::Dynamic => false,
        });
        assert!(holds_itself);
    }

    /// Folding an array of functions into their composition puts in the state a lambda capturing
    /// the state itself, so the set of the result holds a lambda that captures a function of the
    /// set: however long the chain grows at run time, the set has the two lambdas, and the program
    /// asks for one copy.
    #[test]
    fn test_a_fold_composing_functions_asks_for_one_copy_of_a_set_holding_itself() {
        let prg = program_of(
            r#"
module Main;

compose_all : Array (I64 -> I64) -> I64 -> I64;
compose_all = |fs| fs.to_iter.fold(|x| x, |f, acc| |x| f(acc(x)));

main : IO ();
main = (
    let k = *get_args.map(|args| args.@size);
    let fs = [|x| x + k, |x| x * 2];
    println(compose_all(fs)(3).to_string)
);
"#,
        );
        let mut inf = Inferred::new(&prg);
        let sets = inf.sets_at("compose_all", &[1]);
        assert_eq!(sets.len(), 1);
        let s = sets[0];
        assert_eq!(
            inf.writers(s),
            vec![
                Some("compose_all".to_string()),
                Some("compose_all".to_string())
            ]
        );
        let holds_itself = inf.spec.store.elems(s).iter().any(|e| match e {
            Elem::Lam(_, args) => args.contains(&s),
            Elem::Dynamic => false,
        });
        assert!(holds_itself);
    }

    /// A `Std::fix` that `defunctionalize_fix` leaves applies the function it is given to a closure
    /// the combinator builds at run time, so the set of that function holds Dynamic beside the
    /// lambda passed.
    #[test]
    fn test_a_function_given_to_fix_is_dynamic() {
        let prg = program_of(
            r#"
module Main;

run_fixed : ((I64 -> I64) -> I64 -> I64) -> I64 -> I64;
run_fixed = |step, n| fix(step)(n);

main : IO ();
main = (
    let k = 3;
    println(run_fixed(|loop, i| if i == 0 { 0 } else { k * i + loop(i - 1) }, 10).to_string)
);
"#,
        );
        let mut inf = Inferred::new(&prg);
        let sets = inf.sets_at("run_fixed", &[0]);
        assert_eq!(sets.len(), 1);
        assert_eq!(sorted(inf.writers(sets[0])), vec![None, main_lam()]);
    }

    /// The runtime runs the entry `IO` value, and C code calls an exported function through each
    /// arrow its type has, so those sets hold Dynamic. `main` leaves `twice` alone, so that the
    /// export is what asks for every copy of it.
    #[test]
    fn test_the_entry_and_an_exported_value_are_dynamic() {
        let prg = program_of(
            r#"
module Main;

twice : I64 -> I64 -> I64;
twice = |x, y| x * 2 + y;

FFI_EXPORT[twice, fixtwice];

main : IO ();
main = println("exported");
"#,
        );
        let mut inf = Inferred::new(&prg);
        for (name, path) in [("main", vec![]), ("twice", vec![]), ("twice", vec![1])] {
            let sets = inf.sets_at(name, &path);
            assert!(!sets.is_empty());
            for s in sets {
                assert!(inf.writers(s).contains(&None), "{} {:?}", name, path);
            }
        }
    }

    /// A lambda in a field of a struct keeps its set through the getter, the setter of another
    /// field and an `if` choosing between two versions of the struct: the function taking the
    /// struct gets the one lambda put in the field.
    #[test]
    fn test_a_lambda_in_a_struct_field_keeps_its_set() {
        let prg = program_of(
            r#"
module Main;

type Seg = box struct { op : (I64, I64) -> I64, data : Array I64 };

use_seg : Seg -> I64;
use_seg = |s| (s.@op)((s.@data.@(0), 2));

main : IO ();
main = (
    let n = *get_args.map(|args| args.@size);
    let s = Seg { op : |(x, y)| max(x, y), data : [1, 2] };
    let s = if n > 1 { s.set_data([5]) } else { s };
    println(use_seg(s).to_string)
);
"#,
        );
        let mut inf = Inferred::new(&prg);
        let sets = inf.sets_at("use_seg", &[0]);
        assert_eq!(sets.len(), 1);
        assert_eq!(inf.writers(sets[0]), vec![main_lam()]);
    }

    /// A type taking a type constructor without its arguments, given `Array`, tracks the function a
    /// field of each node holds through the array of children the constructor makes: the function
    /// walking the tree gets the two lambdas the program puts in the nodes.
    #[test]
    fn test_a_function_in_a_tree_built_over_a_type_constructor_argument_keeps_its_set() {
        let prg = program_after_inlining(
            r#"
module Main;

type [f : * -> *] Rose f = box struct { kids : f (Rose f), op : I64 -> I64 };

call_rose : Rose Array -> I64 -> I64;
call_rose = |r, x| r.@kids.to_iter.fold((r.@op)(x), |k, acc| acc + call_rose(k, x));

main : IO ();
main = (
    let n = *get_args.map(|args| args.@size);
    let leaf = Rose { kids : [], op : |x| x - n };
    let r = Rose { kids : [leaf], op : |x| x + 100 };
    println(call_rose(r, 5).to_string)
);
"#,
        );
        let mut inf = Inferred::new(&prg);
        let sets = inf.sets_at("call_rose", &[0]);
        assert_eq!(sets.len(), 1);
        assert_eq!(inf.writers(sets[0]), vec![main_lam(), main_lam()]);
    }

    /// A global array of functions that one caller extends with a lambda of its own and another
    /// passes as it is: the function taking the array is copied for each, and the set of the
    /// elements holds the global's lambda in both, beside the caller's in the first.
    #[test]
    fn test_a_global_array_of_functions_extended_by_a_caller_keeps_its_own_lambda() {
        let prg = program_after_inlining(
            r#"
module Main;

table : Array (I64 -> I64);
table = [|x| x + 1000];

run_all : I64 -> Array (I64 -> I64) -> I64 -> I64;
run_all = |d, fs, x| if d > 0 { run_all(d - 1, fs, x) } else { fs.to_iter.fold(0, |f, acc| acc + f(x)) };

main : IO ();
main = (
    let n = *get_args.map(|args| args.@size);
    let a = run_all(n, table.push_back(|x| x * n), 3);
    let b = run_all(n, table, 4);
    println((a + b).to_string)
);
"#,
        );
        let mut inf = Inferred::new(&prg);
        let mut sets: Vec<Vec<Option<String>>> = inf
            .sets_at("run_all", &[1, 0, 0])
            .into_iter()
            .map(|s| sorted(inf.writers(s)))
            .collect();
        sets.sort();
        assert_eq!(
            sets,
            vec![
                vec![main_lam(), Some("table".to_string())],
                vec![Some("table".to_string())],
            ]
        );
    }

    /// A function field rewritten through `mod_`, which punches the field out and plugs the new
    /// value back in, holds both the lambda put in first and the one wrapping it.
    #[test]
    fn test_a_function_field_rewritten_by_mod_holds_the_wrapping_lambda() {
        let prg = program_after_inlining(
            r#"
module Main;

type Seg = box struct { op : I64 -> I64, data : Array I64 };

use_seg : I64 -> Seg -> I64;
use_seg = |d, s| if d > 0 { use_seg(d - 1, s) } else { (s.@op)(s.@data.@(0)) };

main : IO ();
main = (
    let n = *get_args.map(|args| args.@size);
    let s = Seg { op : |x| x + n, data : [1, 2] };
    let s = if n > 1 { s } else { s.mod_op(|f| |x| f(x) * 2) };
    println(use_seg(n, s).to_string)
);
"#,
        );
        let mut inf = Inferred::new(&prg);
        let sets = inf.sets_at("use_seg", &[1, 0]);
        assert_eq!(sets.len(), 1);
        assert_eq!(inf.writers(sets[0]), vec![main_lam(), main_lam()]);
    }

    /// An element of an array of functions rewritten through `Array::mod`, which punches the
    /// element out and plugs the new value back in, leaves the array's elements holding both the
    /// lambda put in first and the one wrapping it.
    #[test]
    fn test_an_array_element_rewritten_by_mod_holds_the_wrapping_lambda() {
        let prg = program_after_inlining(
            r#"
module Main;

run_first : I64 -> Array (I64 -> I64) -> I64;
run_first = |d, fs| if d > 0 { run_first(d - 1, fs) } else { (fs.@(0))(d) };

main : IO ();
main = (
    let n = *get_args.map(|args| args.@size);
    let fs = [|x| x + n];
    let fs = if n > 1 { fs } else { fs.mod(0, |f| |x| f(x) * 2) };
    println(run_first(n, fs).to_string)
);
"#,
        );
        let mut inf = Inferred::new(&prg);
        let sets = inf.sets_at("run_first", &[1, 0, 0]);
        assert_eq!(sets.len(), 1);
        assert_eq!(inf.writers(sets[0]), vec![main_lam(), main_lam()]);
    }
}
