//! The compilation units a program's generated code is divided into, each generated on its own and
//! cached under a hash of everything the code in it is generated from.

use crate::ast::name::FullName;
use crate::constants::COMPILATION_UNITS_PATH;
use std::fmt;
use std::path::PathBuf;

/// A set of the program's RC IR entries — the top-level functions and the global values whose code
/// is generated together — cached under `unit_hash`, so that a rebuild regenerates only the units
/// whose code changed.
pub struct CompileUnit {
    /// The entries this unit generates the code of, ordered by name. Empty for the main unit, which
    /// builds the entry point and the exported C functions instead.
    entries: Vec<FullName>,
    /// The digest naming this unit and the file its object code is cached in. Empty until
    /// `set_unit_hash` sets it.
    unit_hash: String,
}

impl fmt::Display for CompileUnit {
    /// Writes the unit's hash, how many entries it holds, and the name of the first of them.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "CompileUnit(hash = {}, size = {}, entries = [{}, ...])",
            self.unit_hash,
            self.entries.len(),
            match self.entries.first() {
                Some(entry) => entry.to_string(),
                None => "N/A".to_string(),
            },
        )
    }
}

impl CompileUnit {
    /// A unit generating the code of `entries`. `set_unit_hash` gives the unit its hash.
    pub fn new(entries: Vec<FullName>) -> Self {
        CompileUnit {
            entries,
            unit_hash: String::new(),
        }
    }

    /// The digest naming this unit and the file its object code is cached in. Empty until the hash
    /// is set.
    pub fn unit_hash(&self) -> &str {
        &self.unit_hash
    }

    /// The entries this unit generates the code of, ordered by name.
    pub fn entries(&self) -> &[FullName] {
        &self.entries
    }

    /// The path of the object file this unit's generated code is compiled into: `unit_hash` under
    /// `COMPILATION_UNITS_PATH`. A file already there is one an earlier build left, and the build
    /// links it instead of generating the unit again. Requires the unit's hash to be set.
    pub fn object_file_path(&self) -> PathBuf {
        if self.unit_hash.is_empty() {
            panic!("unit_hash is not set.");
        }
        let mut path = PathBuf::from(COMPILATION_UNITS_PATH);
        path.push(format!("{}.o", self.unit_hash));
        path
    }

    /// Names this unit by `hash`, which is what its object file is cached under
    /// (`divide_program::generated_code_hash`). Requires the unit to carry no hash yet.
    pub fn set_unit_hash(&mut self, hash: String) {
        assert!(self.unit_hash.is_empty());
        self.unit_hash = hash;
    }
}
