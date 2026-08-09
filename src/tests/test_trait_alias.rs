// Expanding a trait alias walks the aliases it names down to the traits themselves, and an alias
// that several of them stand for is arrived at once per path leading to it. The walk therefore has
// to remember the aliases it has expanded: without that memory, aliases sharing what they stand for
// are walked once per path, and the number of paths doubles with every level of sharing.

#[cfg(test)]
mod tests {
    use crate::tests::test_util::build_within_and_run;
    use std::time::Duration;

    /// Each level names the level below it twice, so the level below is arrived at along two paths
    /// and the top level along `2^SHARING_LEVELS`. This many levels compiles in under a second
    /// while the walk remembers what it has expanded, and takes minutes while it forgets.
    const SHARING_LEVELS: usize = 24;

    /// Generous next to the second the build takes, with room for a machine several times slower
    /// running the rest of the suite beside it, and far short of what the build costs once every
    /// path is walked.
    const TIMEOUT: Duration = Duration::from_secs(60);

    /// Builds and runs a program whose trait aliases share what they stand for `SHARING_LEVELS`
    /// times over, failing if the build does not finish within `TIMEOUT`.
    ///
    /// The top alias constrains a function the program calls, so the aliases are expanded both
    /// where they are declared and where the constraint is proved.
    #[test]
    fn test_shared_trait_aliases_compile_in_reasonable_time() {
        let mut aliases = String::from("trait Shared0 = ToString;\n");
        for i in 1..=SHARING_LEVELS {
            aliases.push_str(&format!(
                "trait Left{i} = Shared{below};\n\
                 trait Right{i} = Shared{below};\n\
                 trait Shared{i} = Left{i} + Right{i};\n",
                i = i,
                below = i - 1
            ));
        }
        let source = format!(
            "module Main;\n\
             \n\
             {aliases}\n\
             show : [a : Shared{top}] a -> String;\n\
             show = |x| x.to_string;\n\
             \n\
             main : IO ();\n\
             main = println(show(42));\n",
            aliases = aliases,
            top = SHARING_LEVELS
        );

        let printed = build_within_and_run(
            &source,
            "max",
            TIMEOUT,
            &format!("{} levels of shared trait aliases", SHARING_LEVELS),
        );
        assert_eq!(
            printed, "42",
            "the constraint on the shared alias reached a wrong implementation"
        );
    }
}

/// Expanding an alias stands for substituting each alias by what it names until no alias is left,
/// and keeping the first arrival at each trait. This module builds alias graphs of every shape a
/// handful of aliases can take -- diamonds, chains, an alias naming another twice, cycles reachable
/// along one path of several -- and holds `TraitAliasEnv::resolve_alias` to that substitution.
#[cfg(test)]
mod expansion_matches_substitution {
    use crate::ast::name::FullName;
    use crate::ast::traits::{TraitAlias, TraitAliasEnv, TraitId};
    use crate::ast::types::kind_star;
    use crate::misc::{Map, Set};
    use crate::parse::sourcefile::{SourceFile, Span};
    use std::path::PathBuf;

    /// The aliases a generated graph declares, named `A0` to `A5`.
    const ALIASES: usize = 6;

    /// The traits a generated graph declares, named `T0` to `T2`. These are what an alias expands
    /// to, so a generated graph reaches them along however many paths its aliases give.
    const TRAITS: usize = 3;

    /// The most traits and aliases one alias names.
    const MAX_VALUE_LEN: usize = 3;

    /// How many graphs to build. Every shape six aliases naming up to three names each can take is
    /// reached within this many.
    const GRAPHS: usize = 4000;

    /// A xorshift generator, so the graphs are the same on every run.
    struct Rng(u64);

    impl Rng {
        fn next(&mut self) -> u64 {
            self.0 ^= self.0 >> 12;
            self.0 ^= self.0 << 25;
            self.0 ^= self.0 >> 27;
            self.0.wrapping_mul(0x2545F4914F6CDD1D)
        }

        fn below(&mut self, n: usize) -> usize {
            (self.next() % n as u64) as usize
        }
    }

    fn alias_id(i: usize) -> TraitId {
        TraitId::from_fullname(FullName::from_strs(&["Test"], &format!("A{}", i)))
    }

    fn trait_id(i: usize) -> TraitId {
        TraitId::from_fullname(FullName::from_strs(&["Test"], &format!("T{}", i)))
    }

    /// A span over a source file held in memory, so a report a generated graph draws renders.
    fn span() -> Span {
        let source = SourceFile::from_file_path_and_content(
            PathBuf::from("generated_alias_graph.fix"),
            "trait A0 = A1;\n".to_string(),
        );
        Span {
            input: source,
            start: 0,
            end: 14,
        }
    }

    /// An alias graph in which each alias names between one and `MAX_VALUE_LEN` of the aliases and
    /// the traits.
    ///
    /// `standing_only_for_later` restricts an alias to the aliases declared after it, so that no
    /// alias stands for itself however long a chain of them is followed. Lifting the restriction
    /// lets an alias name any of them, itself included, and most such graphs do have an alias that
    /// stands for itself.
    fn generate_graph(rng: &mut Rng, standing_only_for_later: bool) -> TraitAliasEnv {
        let mut data = Map::default();
        for i in 0..ALIASES {
            let first_alias = if standing_only_for_later { i + 1 } else { 0 };
            let mut value = vec![];
            for _ in 0..1 + rng.below(MAX_VALUE_LEN) {
                let picked = first_alias + rng.below(ALIASES + TRAITS - first_alias);
                let named = if picked < ALIASES {
                    alias_id(picked)
                } else {
                    trait_id(picked - ALIASES)
                };
                value.push((named, span()));
            }
            data.insert(
                alias_id(i),
                TraitAlias {
                    id: alias_id(i),
                    value,
                    source: Some(span()),
                    name_src: Some(span()),
                    kind: kind_star(),
                },
            );
        }
        TraitAliasEnv { data }
    }

    /// What `trait_id` stands for once every alias in it is substituted by what that alias names,
    /// with `entered` the aliases already substituted along this branch.
    ///
    /// Answers `None` once a branch has substituted `ALIASES` of them and meets another, since a
    /// branch that long has substituted one of them twice and the substitution does not terminate.
    fn substitute(env: &TraitAliasEnv, trait_id: &TraitId, entered: usize) -> Option<Vec<TraitId>> {
        let Some(alias) = env.data.get(trait_id) else {
            return Some(vec![trait_id.clone()]);
        };
        if entered >= ALIASES {
            return None;
        }
        let mut res = vec![];
        for (named, _) in &alias.value {
            res.extend(substitute(env, named, entered + 1)?);
        }
        Some(res)
    }

    /// `ids` as names, with the arrivals after the first at each name dropped.
    fn first_arrivals(ids: Vec<TraitId>) -> Vec<String> {
        let mut arrived = Set::default();
        ids.into_iter()
            .filter(|id| arrived.insert(id.clone()))
            .map(|id| id.to_string())
            .collect()
    }

    /// `ids` as names, each kept where it stands.
    fn names(ids: Vec<TraitId>) -> Vec<String> {
        ids.into_iter().map(|id| id.to_string()).collect()
    }

    #[test]
    fn test_expansion_matches_substitution() {
        let mut rng = Rng(0x9E3779B97F4A7C15);
        let mut expanded = 0;
        let mut reported = 0;
        for graph_index in 0..2 * GRAPHS {
            let env = generate_graph(&mut rng, graph_index % 2 == 0);
            for i in 0..ALIASES {
                let root = alias_id(i);
                let graph = describe(&env);
                match (env.resolve_alias(&root), substitute(&env, &root, 0)) {
                    (Ok(found), Some(stands_for)) => {
                        expanded += 1;
                        assert_eq!(
                            names(found),
                            first_arrivals(stands_for),
                            "`{}` was expanded into something other than what it stands for, in:\n{}",
                            root.to_string(),
                            graph
                        );
                    }
                    (Err(errs), None) => {
                        reported += 1;
                        let msg = errs.to_string();
                        assert!(
                            msg.contains("Circular aliasing detected in trait alias `Test::A"),
                            "expanding `{}` was refused for a reason other than circular aliasing, \
                             or the report named a trait rather than an alias:\n{}\nin:\n{}",
                            root.to_string(),
                            msg,
                            graph
                        );
                    }
                    (Ok(found), None) => panic!(
                        "`{}` stands for itself, and was expanded into [{}] all the same, in:\n{}",
                        root.to_string(),
                        names(found).join(", "),
                        graph
                    ),
                    (Err(errs), Some(stands_for)) => panic!(
                        "`{}` stands for [{}], and expanding it was refused, in:\n{}\n{}",
                        root.to_string(),
                        first_arrivals(stands_for).join(", "),
                        graph,
                        errs.to_string()
                    ),
                }
            }
        }
        assert!(
            expanded >= GRAPHS && reported >= GRAPHS,
            "the graphs reached one of the two answers too rarely to hold the expansion to \
             anything: {} expanded, {} reported",
            expanded,
            reported
        );
    }

    /// The graph as the declarations that would produce it, for a failure to name.
    fn describe(env: &TraitAliasEnv) -> String {
        let mut res = String::new();
        for i in 0..ALIASES {
            let alias = env.data.get(&alias_id(i)).unwrap();
            let named: Vec<String> = alias.value.iter().map(|(id, _)| id.to_string()).collect();
            res += &format!("trait {} = {};\n", alias.id.to_string(), named.join(" + "));
        }
        res
    }
}
