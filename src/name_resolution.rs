use std::sync::Arc;

use crate::{
    ast::{
        import::{self, ImportStatement},
        name::{FullName, Name},
        program::ModuleInfo,
    },
    constants::ERR_UNKNOWN_NAME,
    error::{Error, Errors},
    misc::{self, Map, Set},
    sourcefile::Span,
};

// Environment for name resolution.
pub struct NameResolutionEnv {
    pub candidates: Map<FullName, NameResolutionType>,
    pub assoc_ty_to_arity: Map<FullName, usize>,
    mod_to_import_stmts: Map<Name, Vec<ImportStatement>>,
    module_infos: Vec<ModuleInfo>,
}

impl NameResolutionEnv {
    pub fn new(
        tycon_names_with_aliases: &Set<FullName>,
        trait_names_with_aliases: &Set<FullName>,
        assoc_ty_to_arity: Map<FullName, usize>,
        mod_to_import_stmts: Map<Name, Vec<ImportStatement>>,
        module_infos: Vec<ModuleInfo>,
    ) -> Self {
        let mut candidates: Map<FullName, NameResolutionType> = Map::default();
        fn check_insert(
            candidates: &mut Map<FullName, NameResolutionType>,
            name: FullName,
            nrt: NameResolutionType,
        ) {
            assert!(!candidates.contains_key(&name) || candidates[&name] == nrt); // This is assured by `validate_capital_name_confliction`.
            candidates.insert(name, nrt);
        }
        for name in tycon_names_with_aliases {
            check_insert(&mut candidates, name.clone(), NameResolutionType::TyCon);
        }
        for name in trait_names_with_aliases {
            check_insert(&mut candidates, name.clone(), NameResolutionType::Trait);
        }
        for (name, _arity) in &assoc_ty_to_arity {
            check_insert(&mut candidates, name.clone(), NameResolutionType::AssocTy);
        }
        NameResolutionEnv {
            candidates,
            assoc_ty_to_arity,
            mod_to_import_stmts,
            module_infos,
        }
    }
}

pub struct NameResolutionContext {
    // The current module where name resolution is performed.
    current_module: Name,
    // The name resolution environment.
    pub env: Arc<NameResolutionEnv>,
    // The list of names that should be imported in each module
    // This is mutated during name resolution.
    pub import_required: Map<Name, Vec<FullName>>,
}

impl NameResolutionContext {
    pub fn valid_import_stmts(&self) -> &Vec<ImportStatement> {
        match self.env.mod_to_import_stmts.get(&self.current_module) {
            Some(stmts) => stmts,
            None => panic!(
                "Module {:?} not found in import statements map.",
                self.current_module
            ),
        }
    }

    pub fn current_module_info(&self) -> &ModuleInfo {
        for mi in &self.env.module_infos {
            if &mi.name == &self.current_module {
                return mi;
            }
        }
        panic!(
            "Module {:?} not found in module infos.",
            self.current_module
        );
    }

    pub fn new(current_module: Name, env: Arc<NameResolutionEnv>) -> Self {
        NameResolutionContext {
            current_module,
            env,
            import_required: Map::default(),
        }
    }

    pub fn set_current_module(&mut self, module: Name) {
        self.current_module = module;
    }

    pub fn add_import_required(&mut self, names: Vec<FullName>) {
        misc::insert_to_map_vec_many(&mut self.import_required, &self.current_module, names);
    }

    // Resolve the given short name to a full name.
    //
    // If there are multiple candidates, or no candidates, return an error.
    //
    // If the `short_name` is an absolute name, the returned full name will also be set as an absolute name.
    pub fn resolve(
        &mut self,
        short_name: &FullName,
        accept_types: &[NameResolutionType],
        span: &Option<Span>,
    ) -> Result<FullName, Errors> {
        let accept_type_string = accept_types
            .iter()
            .map(|nrt| nrt.to_string())
            .collect::<Vec<_>>()
            .join(" or ");
        let candidates = self
            .env
            .candidates
            .iter()
            .filter_map(|(full_name, nrt)| {
                if !accept_types.contains(nrt) {
                    return None;
                }
                if !short_name.is_suffix_of(full_name) {
                    return None;
                }
                // Absolute name are accepted without checking accessibility.
                let inaccessible = !short_name.is_absolute()
                    && !import::is_accessible(&self.valid_import_stmts(), full_name);
                if inaccessible {
                    return None;
                }
                // Inherit the abosolute property from the short name.
                let mut full_name = full_name.clone();
                if short_name.is_absolute() {
                    full_name.set_absolute();
                }
                return Some(full_name);
            })
            .collect::<Vec<_>>();
        if candidates.len() == 0 {
            let src = span
                .clone()
                .unwrap_or(self.current_module_info().source.clone());
            let mut err = Error::from_msg_srcs(
                format!(
                    "Unknown {} name `{}`.",
                    accept_type_string,
                    short_name.to_string()
                ),
                &[&Some(src)],
            );
            err.code = Some(ERR_UNKNOWN_NAME);
            err.data = Some(serde_json::Value::String(short_name.to_string()));
            Err(Errors::from_err(err))
        } else if candidates.len() == 1 {
            let full_name = candidates[0].clone();
            if !full_name.is_absolute() {
                misc::insert_to_map_vec(
                    &mut self.import_required,
                    &self.current_module,
                    full_name.clone(),
                );
            }
            Ok(full_name)
        } else {
            // candidates.len() >= 2
            let msg = NameResolutionContext::create_ambiguous_message(
                &short_name.to_string(),
                candidates,
                false,
            );
            Err(Errors::from_msg_srcs(msg, &[span]))
        }
    }

    pub fn create_ambiguous_message(
        short_name: &str,
        mut candidates: Vec<FullName>,
        add_type_annotation: bool,
    ) -> String {
        candidates.sort(); // Sort for deterministic error message.

        // Join the candidates with ", ".
        let candidates_str = candidates
            .iter()
            .map(|fullname| "`".to_string() + &fullname.to_string() + "`")
            .collect::<Vec<_>>()
            .join(", ");

        // The Error message.
        let mut msg = format!(
            "Name `{}` is ambiguous: there are {}. Add (a suffix of) its namespace{} to help overloading resolution.",
            short_name,
            candidates_str,
            if add_type_annotation { " or type annotation" } else { "" }
        );

        // Check if there is candidates (x, y) such that x is a suffix of y.
        let mut suffixes = vec![];
        for i in 0..candidates.len() {
            for j in 0..candidates.len() {
                if i != j
                    && candidates[i]
                        .namespace
                        .is_suffix_of(&candidates[j].namespace)
                {
                    suffixes.push(candidates[i].clone());
                }
            }
        }
        // If there are suffixes, notify the user that they can use absolute namespace.
        if suffixes.len() > 0 {
            msg += &format!(
                " Here, you need to use absolute namespaces to specify {}; i.e., write as {}.",
                suffixes
                    .iter()
                    .map(|fullname| format!("`{}`", fullname.to_string()))
                    .collect::<Vec<_>>()
                    .join(", "),
                suffixes
                    .iter()
                    .map(|fullname| format!("`::{}`", fullname.to_string()))
                    .collect::<Vec<_>>()
                    .join(", "),
            );
        }

        msg
    }
}

#[derive(PartialEq, Eq)]
pub enum NameResolutionType {
    TyCon,
    Trait,
    AssocTy,
}

impl NameResolutionType {
    pub fn to_string(&self) -> &'static str {
        match self {
            NameResolutionType::TyCon => "type",
            NameResolutionType::Trait => "trait",
            NameResolutionType::AssocTy => "associated type",
        }
    }
}
