// Lambda set specialization: which lambdas a value of function type can be, carried in its type.
//
// Every slot of a type where a function value sits (an arrow, and the arrows a struct, a union or an
// array declares) carries a lambda set variable. Inference unifies the variables of two places whose
// values flow into each other, as the type checker unifies their types, and records each lambda in
// the variable of the place it is built at. A component of the call graph is inferred together and
// generalized over the variables of its symbols' types, so that a function taking a function is
// inferred once and copied for each lambda set it is given. Specialization then walks from the
// program's entry, and gives each copy the concrete lambda sets of its parameters. The program is
// then rewritten: each copy becomes copies of the symbols, and a value whose set's elements make
// one class, elements naming one lambda that make the same code, becomes that class's capture
// struct, called by calling the lambda directly. A value whose set's elements make two or more
// classes becomes an unboxed union of their capture structs, called by matching the variant and
// calling its class's lambda directly. A set no capture struct or union can stand for stays a set of
// closures (`represent::represent` lists them).
//
// The design is laid out in `dev-docs/2026-10-24-lambda-set-specialization/design.md`.

mod concrete;
mod dump;
mod flow;
mod infer;
mod partition;
mod represent;
mod rewrite;
mod slots;
mod solver;
mod specialize;

use crate::{
    ast::{program::Program, type_agreement::validate_types},
    constants::DOT_FIXLANG,
    error::panic_if_err,
    project_fs::{write_file_guarding_project_links, DOT_FIXLANG_LINK_HINT},
};
pub use flow::BuiltinFlow;
use infer::infer;
use represent::represent;
use rewrite::rewrite;
use slots::SlotLayout;
use specialize::specialize;
use std::{fs, path::PathBuf};

/// Infers the lambda sets of `prg` and the copies they ask for, and rewrites the program by them.
/// The bodies of the lifted lambdas are moved into their callers on up to `num_threads` threads.
/// With `dump_step`, writes the sets and the copies to a file under `.fixlang` named after it; a
/// failure to write it ends the process with the error printed. With `develop_mode`, checks that
/// the types of the rewritten program agree where its expressions meet.
pub fn run(prg: &mut Program, dump_step: Option<String>, num_threads: usize, develop_mode: bool) {
    let rewritten = {
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
        let repr = represent(&analysis, &spec);
        rewrite(prg, &layout, &analysis, &spec, &repr)
    };
    rewritten.apply(prg, num_threads);
    if develop_mode {
        validate_types(prg);
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
        ast::program::Program, elaboration::elaborate_at_max, optimization::unwrap_newtype,
    };

    /// `source` elaborated into the form the inference reads: monomorphic, curried, and with its
    /// newtypes unwrapped.
    fn program_of(source: &str) -> Program {
        let (mut prg, _) = elaborate_at_max(source, "lambda_set", 0);
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
        let holds_itself = inf.spec.store.lam_elems(s).any(|(lam, args)| {
            inf.analysis.lambdas[lam as usize]
                .symbol
                .name
                .starts_with("rec")
                && args.contains(&s)
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
        let holds_itself = inf
            .spec
            .store
            .lam_elems(s)
            .any(|(_, args)| args.contains(&s));
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

    /// A lambda capturing a struct depends on every set the struct's type carries, the sets of the
    /// fields its body leaves unread included: the struct is part of its capture. Here the body
    /// reads the field `f` of the captured `p` only through the getter, which inlining leaves as a
    /// builtin reading `p` by name, and the field `g` not at all.
    #[test]
    fn test_a_lambda_depends_on_the_sets_of_a_struct_it_captures() {
        let prg = program_after_inlining(
            r#"
module Main;

type P = box struct { f : I64 -> I64, g : I64 -> I64 };

mk : I64 -> P -> I64 -> I64;
mk = |d, p| if d > 0 { mk(d - 1, p) } else { |x| (p.@f)(x) };

main : IO ();
main = (
    let n = *get_args.map(|args| args.@size);
    let f = |x| x + n;
    let a = mk(n, P { f : f, g : |x| x + 2 })(1);
    let b = mk(n, P { f : f, g : |x| x * 3 })(1);
    println((a + b).to_string)
);
"#,
        );
        let mut inf = Inferred::new(&prg);
        let (c, _) = inf.component_of("mk");
        let ty = inf.type_of("mk");
        let p_ty = inf.layout.type_arg(&inf.layout.type_arg(&ty, 1), 0);
        let p_slots: Vec<usize> = inf
            .layout
            .own_slots(&p_ty)
            .to_vec()
            .iter()
            .map(|v| {
                let root = inf.analysis.solver.find(*v);
                inf.analysis.components[c]
                    .params
                    .iter()
                    .position(|p| *p == root)
                    .unwrap()
            })
            .collect();
        assert_eq!(p_slots.len(), 2);
        let inner = inf
            .analysis
            .lambdas
            .iter()
            .find(|l| l.symbol.name.starts_with("mk") && l.index == 2)
            .unwrap();
        for p in p_slots {
            assert!(
                inner.params.contains(&p),
                "the lambda capturing `p` depends on {:?}, missing parameter {}",
                inner.params,
                p
            );
        }
    }

    /// The source of a chain of `depth` symbols over `both`, which builds a value from two: each
    /// symbol of the chain refers twice to the one below. With `of_functions` the symbols are
    /// functions of a `P` passing it down, otherwise global values. `both` and each function of
    /// the chain recurse, so that no inlining collapses the chain.
    fn doubling_chain(depth: usize, of_functions: bool) -> String {
        let mut source = String::from(
            "module Main;\n\
             type P = box struct { run : I64 -> I64 };\n\
             both : P -> P -> P;\n\
             both = |a, b| if (a.@run)(0) < -1000000 { both(b, a) } \
             else { P { run : |x| (a.@run)((b.@run)(x)) } };\n",
        );
        if of_functions {
            source += "c0 : P -> P;\n\
                       c0 = |a| if (a.@run)(0) < -1000000 { c0(a) } \
                       else { P { run : |x| (a.@run)(x) + 1 } };\n";
            for i in 1..=depth {
                source += &format!(
                    "c{i} : P -> P;\n\
                     c{i} = |a| if (a.@run)(0) < -1000000 {{ c{i}(a) }} \
                     else {{ both(c{j}(a), c{j}(a)) }};\n",
                    j = i - 1
                );
            }
            source += &format!(
                "main : IO ();\nmain = (let k = *get_args.map(|args| args.@size); \
                 println((c{depth}(P {{ run : |x| x + k }}).@run)(k).to_string));\n"
            );
        } else {
            source += "c0 : P;\nc0 = P { run : |x| x + 1 };\n";
            for i in 1..=depth {
                source += &format!("c{i} : P;\nc{i} = both(c{j}, c{j});\n", j = i - 1);
            }
            source += &format!(
                "main : IO ();\nmain = (let k = *get_args.map(|args| args.@size); \
                 println((c{depth}.@run)(k).to_string));\n"
            );
        }
        source
    }

    /// A symbol referring twice to one whose scheme has internal classes gets the two copies of
    /// them merged into one, so that along a chain of such symbols the scheme of each stays as
    /// small as the one below rather than doubling: for global values, and for functions whose
    /// internal classes depend on their parameter.
    #[test]
    fn test_a_chain_of_symbols_referring_twice_to_the_one_below_keeps_its_schemes_small() {
        for of_functions in [false, true] {
            let prg = program_of(&doubling_chain(12, of_functions));
            let inf = Inferred::new(&prg);
            let (c, _) = inf.component_of("c12");
            let classes = inf.analysis.components[c].scheme.classes.len();
            assert!(
                classes <= 16,
                "the scheme at the top of the chain of {} has {} classes",
                if of_functions {
                    "functions"
                } else {
                    "global values"
                },
                classes
            );
        }
    }
}
