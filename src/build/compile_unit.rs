//! The division of a program into compilation units, each generated on its own and cached under a
//! hash of everything the code in it is generated from.

use crate::ast::name::Name;
use crate::ast::program::Symbol;
use crate::configuration::Configuration;
use crate::constants::COMPILATION_UNITS_PATH;
use crate::hash::{md5_hex, HashSource};
use crate::misc::{split_at_name_boundaries, Map, Set};
use rand::Rng;
use std::fmt;
use std::path::PathBuf;

/// What a compilation unit's code generation leaves on disk for the link step, as
/// `Configuration::unit_output` chooses it.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum UnitOutput {
    /// An object file, optimized on its own, which the linker puts together with the other units'.
    ObjectFile,
    /// LLVM bitcode carrying the unit's code as generation left it, to be merged with the other
    /// units' into one module that is optimized and compiled as a whole.
    Bitcode,
}

impl UnitOutput {
    /// The extension naming the file this output is written to.
    fn extension(self) -> &'static str {
        match self {
            UnitOutput::ObjectFile => "o",
            UnitOutput::Bitcode => "bc",
        }
    }
}

/// A set of the program's symbols generated together and cached under `unit_hash`, so that a
/// rebuild regenerates only the units whose inputs changed.
pub struct CompileUnit {
    /// The symbols this unit compiles.
    symbols: Vec<Symbol>,
    /// The modules whose source decides the code generated for `symbols`. A change to any of them
    /// invalidates the unit.
    dependent_modules: Vec<Name>,
    /// The digest naming this unit and the file its object code is cached in. Empty until
    /// `update_unit_hash` or `set_random_unit_hash` sets it.
    unit_hash: String,
}

impl fmt::Display for CompileUnit {
    /// Writes the unit's hash, how many symbols it holds and the name of the first of them, and the
    /// modules it depends on.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "CompileUnit(hash = {}, size = {}, symbols = [{}, ...], dependency = [{}])",
            self.unit_hash,
            self.symbols.len(),
            if self.symbols.len() > 0 {
                self.symbols[0].name.to_string()
            } else {
                "N/A".to_string()
            },
            self.dependent_modules.join(", "),
        )
    }
}

impl CompileUnit {
    /// A unit compiling `symbols`, whose generated code is decided by the source of
    /// `dependent_modules`. `update_unit_hash` gives the unit its hash.
    pub fn new(symbols: Vec<Symbol>, dependent_modules: Vec<Name>) -> Self {
        CompileUnit {
            symbols,
            dependent_modules,
            unit_hash: "".to_string(),
        }
    }

    /// The digest naming this unit and the file its object code is cached in. Empty until the hash
    /// is set.
    pub fn unit_hash(&self) -> &str {
        &self.unit_hash
    }

    /// The symbols this unit compiles. Taking the unit's hash sorts them by name.
    pub fn symbols(&self) -> &[Symbol] {
        &self.symbols
    }

    /// The path of the file this unit's generated code is cached in: `unit_hash` under
    /// `COMPILATION_UNITS_PATH`, with the extension `output` names. A file already there is one an
    /// earlier build left, and the build takes it instead of generating the unit again. Requires
    /// the unit's hash to be set.
    pub fn output_file_path(&self, output: UnitOutput) -> PathBuf {
        if self.unit_hash.len() == 0 {
            panic!("unit_hash is not set.");
        }
        let mut path = PathBuf::from(COMPILATION_UNITS_PATH);
        path.push(format!("{}.{}", self.unit_hash, output.extension()));
        path
    }

    /// Takes this unit's hash over the configuration, the symbols and the hashes of the dependent
    /// modules, and sets it on `self`. A unit that already carries a hash keeps it.
    ///
    /// # Arguments
    /// * `module_dependency_hash` - for each module, a digest of everything a value defined in it
    ///   is compiled from, as `Program::module_dependency_hash_map` gives it. Every module this
    ///   unit depends on has an entry.
    pub fn update_unit_hash(
        &mut self,
        module_dependency_hash: &Map<Name, String>,
        config: &Configuration,
    ) {
        if self.unit_hash.len() > 0 {
            return;
        }

        self.symbols
            .sort_by(|a, b| a.name.to_string().cmp(&b.name.to_string()));
        self.dependent_modules.sort();

        let mut hash_source = HashSource::default();

        // The settings the code of this unit is generated under.
        hash_source.push_text(&config.object_generation_hash());

        // The symbols this unit implements.
        hash_source.push_list(self.symbols.iter().map(Symbol::hash));

        // The sources of the modules this unit's symbols are made of.
        hash_source.push_list(
            self.dependent_modules
                .iter()
                .map(|name| &module_dependency_hash[name]),
        );

        self.unit_hash = hash_source.finish();
    }

    /// Sets this unit's hash to a random value, so that the build generates the unit's object file
    /// afresh however the cache stands. Requires the unit to carry no hash yet.
    #[allow(dead_code)]
    pub fn set_random_unit_hash(&mut self) {
        assert!(self.unit_hash.len() == 0);
        self.unit_hash = md5_hex(&rand::thread_rng().gen::<u64>().to_string());
    }

    /// Divides this unit into units averaging `mean_size` symbols each, every one of them
    /// depending on the modules this one depends on. Requires this unit to carry no hash yet,
    /// since each piece takes a hash of its own.
    pub fn split_at_name_boundaries(self, mean_size: usize) -> Vec<CompileUnit> {
        // `unit_hash` is lost after this method is called.
        assert_eq!(self.unit_hash, "");

        let symbols = self.symbols;
        let dependent_modules = self.dependent_modules;

        // The text naming a symbol is its `name` alone, so that editing a symbol's `expr` leaves
        // every boundary where it was and the units holding no edited symbol keep their cached
        // object files. `Symbol::hash` takes in the `expr` too, so a boundary read off it would
        // move at every symbol whose `expr` changed.
        let symbol_pieces =
            split_at_name_boundaries(symbols, mean_size, |symbol| symbol.name.to_string());
        let mut units = vec![];
        for piece in symbol_pieces {
            units.push(CompileUnit::new(piece, dependent_modules.clone()));
        }

        units
    }

    /// Divides `symbols` into compilation units, each carrying its hash.
    ///
    /// Symbols sharing a set of dependent modules go into one unit, which is then divided into
    /// units averaging `config.cu_size` symbols.
    pub fn split_symbols(
        symbols: Vec<Symbol>,
        module_dependency_hash: &Map<Name, String>,
        module_dependency_map: &Map<Name, Set<Name>>,
        config: &Configuration,
    ) -> Vec<CompileUnit> {
        let mut units_by_dependencies: Map<
            String, /* concatenated string of dependent modules sorted by their names */
            CompileUnit,
        > = Map::default();
        // Classify symbols into compilation units depending on their dependent modules.
        for symbol in symbols {
            let mut dependent_modules = Set::default();
            for module in symbol.dependent_modules() {
                dependent_modules.extend(module_dependency_map[&module].clone());
            }
            let mut dependent_modules = dependent_modules.iter().cloned().collect::<Vec<_>>();
            dependent_modules.sort();
            let dependent_modules_key = dependent_modules.join(", ");
            let unit = if let Some(unit) = units_by_dependencies.get_mut(&dependent_modules_key) {
                unit
            } else {
                units_by_dependencies.insert(
                    dependent_modules_key.clone(),
                    CompileUnit::new(vec![], dependent_modules),
                );
                units_by_dependencies
                    .get_mut(&dependent_modules_key)
                    .unwrap()
            };
            unit.symbols.push(symbol);
        }
        let mut units = units_by_dependencies
            .into_iter()
            .map(|(_, unit)| unit)
            .collect::<Vec<_>>();
        for unit in &mut units {
            unit.symbols
                .sort_by(|a, b| a.name.to_string().partial_cmp(&b.name.to_string()).unwrap());
        }

        // Split compilation units into smaller ones if they are too large.
        let mut units = units
            .into_iter()
            .flat_map(|unit| unit.split_at_name_boundaries(config.cu_size))
            .collect::<Vec<_>>();

        // Set unit hash.
        for unit in &mut units {
            unit.update_unit_hash(module_dependency_hash, config);
        }

        units
    }
}

/// The digest naming the object file the merged units are compiled into. It is taken over the
/// units' hashes, and a unit's hash covers everything its code is generated from, so two builds
/// share this digest exactly when they would merge the same bitcode.
///
/// The hashes are sorted, so the digest names the set of units rather than the order they are
/// merged in. Two orders of one set produce object files that hold the same code laid out
/// differently, and either of them serves the build that asked for the other.
pub fn merged_units_hash(units: &[CompileUnit]) -> String {
    let mut unit_hashes = units
        .iter()
        .map(|unit| unit.unit_hash())
        .collect::<Vec<_>>();
    unit_hashes.sort();
    let mut hash_source = HashSource::default();
    hash_source.push_list(unit_hashes);
    hash_source.finish()
}

/// The path of the file the object code compiled from the merged units is cached in: the digest
/// `merged_units_hash` gives them, with the extension `.o`, under `COMPILATION_UNITS_PATH`. A file
/// already there is one an earlier build left, and the build links it instead of generating and
/// merging the units again.
pub fn merged_object_file_path(merged_units_hash: &str) -> PathBuf {
    let mut path = PathBuf::from(COMPILATION_UNITS_PATH);
    path.push(format!("{}.o", merged_units_hash));
    path
}

#[cfg(test)]
mod tests {
    use super::CompileUnit;
    use crate::ast::expr::expr_var;
    use crate::ast::name::FullName;
    use crate::ast::program::Symbol;
    use crate::fixstd::builtin::make_i64_ty;

    /// A symbol named `Std::Test::value#{index}`, whose expression is the variable `body`.
    fn symbol(index: usize, body: &str) -> Symbol {
        let name = FullName::from_strs(&["Std", "Test"], &format!("value#{:04}", index));
        Symbol {
            name: name.clone(),
            generic_name: name,
            ty: make_i64_ty(),
            expr: Some(expr_var(FullName::local(body), None)),
            inline_into_callers: false,
        }
    }

    /// Where a unit ends is decided by the names of the symbols, so editing a symbol's `expr`
    /// leaves every boundary where it was and a unit holding no edited symbol keeps the object file
    /// it was compiled into. A boundary read off `Symbol::hash`, which takes in the `expr` as well,
    /// moves at every symbol whose `expr` changed.
    #[test]
    fn test_split_at_name_boundaries_places_the_boundaries_by_the_symbol_names() {
        const SYMBOL_COUNT: usize = 200;
        const MAX_SIZE: usize = 8;

        let unit_symbol_names = |body: &str| -> Vec<Vec<String>> {
            let symbols = (0..SYMBOL_COUNT).map(|i| symbol(i, body)).collect();
            CompileUnit::new(symbols, vec![])
                .split_at_name_boundaries(MAX_SIZE)
                .iter()
                .map(|unit| unit.symbols().iter().map(|s| s.name.to_string()).collect())
                .collect()
        };

        let before = unit_symbol_names("before");
        assert!(
            before.len() > 1,
            "{} symbols fell into a single unit, which holds no boundary to move",
            SYMBOL_COUNT
        );
        assert_eq!(
            before,
            unit_symbol_names("after"),
            "editing the expression of every symbol moved the unit boundaries"
        );
    }
}
