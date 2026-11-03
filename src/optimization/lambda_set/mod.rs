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

pub use flow::BuiltinFlow;

use crate::{ast::program::Program, constants::DOT_FIXLANG};
use slots::SlotLayout;
use std::{fs::File, io::Write, path::PathBuf};

/// Infers the lambda sets of `prg` and the copies they ask for, leaving the program as it is. With
/// `dump_step`, writes them to a file under `.fixlang` named after it.
pub fn run(prg: &Program, dump_step: Option<String>) {
    let layout = SlotLayout::new(&prg.type_env);
    let mut analysis = infer::infer(prg, &layout);
    let spec = specialize::specialize(&mut analysis);
    if let Some(step) = dump_step {
        let text = dump::render(&mut analysis, &spec, &layout);
        let path = PathBuf::from(format!("{}/{}.lambda_sets.txt", DOT_FIXLANG, step));
        let mut file = File::create(&path).unwrap();
        file.write_all(text.as_bytes()).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::{
        concrete::{Elem, SetId},
        infer::{self, Analysis},
        slots::{AType, SlotLayout},
        specialize::{self, Specialization},
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

    /// The lambda sets of a program and the copies they ask for.
    struct Inferred<'p> {
        prg: &'p Program,
        layout: SlotLayout<'p>,
        analysis: Analysis,
        spec: Specialization,
    }

    impl<'p> Inferred<'p> {
        fn new(prg: &'p Program) -> Self {
            let layout = SlotLayout::new(&prg.type_env);
            let mut analysis = infer::infer(prg, &layout);
            let spec = specialize::specialize(&mut analysis);
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

    fn sorted(mut v: Vec<Option<String>>) -> Vec<Option<String>> {
        v.sort();
        v
    }

    fn main_lam() -> Option<String> {
        Some("main".to_string())
    }

    // A function given a different lambda at each of two calls gets a copy for each, and in each
    // the function parameter's set holds the one lambda that call passes.
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

    // A recursion that passes itself a lambda wrapping the function it was given puts that lambda
    // in the same set as the one from outside: the program asks for one copy, and the set of that
    // copy holds the wrapping lambda, which captures a function of the set itself.
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

    // The runtime runs the entry `IO` value, and C code calls an exported function through each
    // arrow its type has, so those sets hold Dynamic. `main` leaves `twice` alone, so that the
    // export is what asks for every copy of it.
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

    // A lambda in a field of a struct keeps its set through the getter, the setter of another
    // field and an `if` choosing between two versions of the struct: the function taking the
    // struct gets the one lambda put in the field.
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
}
