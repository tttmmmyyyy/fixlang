//! The division of a program into compilation units, each compiled into an object file that is
//! cached under a hash of everything the code in it is generated from.

use crate::misc::{split_by_max_size, Map, Set};
use std::fmt;
use std::path::PathBuf;
use rand::Rng;
use crate::ast::name::Name;
use crate::ast::program::Symbol;
use crate::configuration::Configuration;
use crate::constants::COMPILATION_UNITS_PATH;
use crate::hash::md5_hex;

/// A set of the program's symbols compiled into one object file, which is cached under
/// `unit_hash`, so that a rebuild regenerates only the units whose inputs changed.
///
/// `Configuration::enable_separated_compilation` decides when the program is divided into several
/// units; above that, the whole program is one unit.
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
    /// Writes the unit's hash, how many symbols it holds and the name of the first of them, the
    /// modules it depends on, and whether its object file is cached.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "CompileUnit(hash = {}, size = {}, symbols = [{}, ...], dependency = [{}], is_cached = {})",
            self.unit_hash,
            self.symbols.len(),
            if self.symbols.len() > 0 {
                self.symbols[0].name.to_string()
            } else {
                "N/A".to_string()
            },
            self.dependent_modules.join(", "),
            self.is_cached()
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

    /// Whether this unit's object file is already on disk from an earlier build, so that the build
    /// links that file and skips generating the unit's code.
    pub fn is_cached(&self) -> bool {
        self.object_file_path().exists()
    }

    /// The path of the file this unit's object code is cached in: `unit_hash` with the extension
    /// `.o`, under `COMPILATION_UNITS_PATH`. Requires the unit's hash to be set.
    pub fn object_file_path(&self) -> PathBuf {
        if self.unit_hash.len() == 0 {
            panic!("unit_hash is not set.");
        }
        let mut path = PathBuf::from(COMPILATION_UNITS_PATH);
        let file_name = self.unit_hash.to_string() + ".o";
        path.push(file_name);
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

        // Add dependency to the configuration.
        let mut hash_inputs = vec![];
        hash_inputs.push("<configuration>".to_string());
        hash_inputs.push(config.object_generation_hash());

        // Add dependency to the symbols.
        hash_inputs.push("<symbols>".to_string());
        for symbol in &self.symbols {
            hash_inputs.push(symbol.hash());
        }

        // Add dependency to source codes of the dependent modules.
        hash_inputs.push("<dependent modules>".to_string());
        for name in &self.dependent_modules {
            hash_inputs.push(module_dependency_hash[name].clone());
        }

        self.unit_hash = md5_hex(&hash_inputs.join(", "));
    }

    /// Sets this unit's hash to a random value, so that the build generates the unit's object file
    /// afresh however the cache stands. Requires the unit to carry no hash yet.
    #[allow(dead_code)]
    pub fn set_random_unit_hash(&mut self) {
        assert!(self.unit_hash.len() == 0);
        self.unit_hash = md5_hex(&rand::thread_rng().gen::<u64>().to_string());
    }

    /// Divides this unit into units of at most `max_size` symbols each, every one of them
    /// depending on the modules this one depends on. Requires this unit to carry no hash yet,
    /// since each piece takes a hash of its own.
    pub fn split_by_max_size(self, max_size: usize) -> Vec<CompileUnit> {
        // `unit_hash` is lost after this method is called.
        assert_eq!(self.unit_hash, "");

        let symbols = self.symbols;
        let dependent_modules = self.dependent_modules;

        // The text naming a symbol is its `name` alone, so that editing a symbol's `expr` leaves
        // every boundary where it was and the units holding no edited symbol keep their cached
        // object files. `Symbol::hash` takes in the `expr` too, so a boundary read off it would
        // move at every symbol whose `expr` changed.
        let symbol_pieces = split_by_max_size(symbols, max_size, |symbol| symbol.name.to_string());
        let mut units = vec![];
        for piece in symbol_pieces {
            units.push(CompileUnit::new(piece, dependent_modules.clone()));
        }

        units
    }

    /// Divides `symbols` into compilation units, each carrying its hash.
    ///
    /// Symbols sharing a set of dependent modules go into one unit, and a unit holding more than
    /// `config.max_cu_size` symbols is divided further.
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
            .flat_map(|unit| unit.split_by_max_size(config.max_cu_size))
            .collect::<Vec<_>>();

        // Set unit hash.
        for unit in &mut units {
            unit.update_unit_hash(module_dependency_hash, config);
        }

        units
    }
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
    fn test_split_by_max_size_places_the_boundaries_by_the_symbol_names() {
        const SYMBOL_COUNT: usize = 200;
        const MAX_SIZE: usize = 8;

        let unit_symbol_names = |body: &str| -> Vec<Vec<String>> {
            let symbols = (0..SYMBOL_COUNT).map(|i| symbol(i, body)).collect();
            CompileUnit::new(symbols, vec![])
                .split_by_max_size(MAX_SIZE)
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
