use crate::{
    ast::{
        import::{self, ImportStatement},
        name::FullName,
        program::ModuleInfo,
    },
    constants::ERR_UNKNOWN_NAME,
    error::{Error, Errors},
    misc::{Map, Set},
    sourcefile::Span,
};

pub struct NameResolutionContext {
    // The current module where name resolution is performed.
    // This field is used to generate better error messages.
    pub current_module: Option<ModuleInfo>,
    pub candidates: Map<FullName, NameResolutionType>,
    pub assoc_ty_to_arity: Map<FullName, usize>,
    pub import_statements: Vec<ImportStatement>,
}

impl NameResolutionContext {
    pub fn new(
        current_module: Option<ModuleInfo>,
        tycon_names_with_aliases: &Set<FullName>,
        trait_names_with_aliases: &Set<FullName>,
        assoc_ty_to_arity: Map<FullName, usize>,
        import_statements: Vec<ImportStatement>,
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
        NameResolutionContext {
            current_module,
            candidates,
            import_statements,
            assoc_ty_to_arity,
        }
    }

    // Resolve the given short name to a full name.
    //
    // If there are multiple candidates, or no candidates, return an error.
    //
    // If the `short_name` is an absolute name, the returned full name will also be set as an absolute name.
    pub fn resolve(
        &self,
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
            .candidates
            .iter()
            .filter_map(|(full_name, nrt)| {
                if !accept_types.contains(nrt) {
                    return None;
                }
                if !short_name.is_suffix(full_name) {
                    return None;
                }
                // Absolute name are accepted even if it is not imported.
                let inaccessible = !short_name.is_absolute()
                    && !import::is_accessible(&self.import_statements, full_name);
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
                .or(self.current_module.as_ref().map(|m| m.source.clone()));
            let mut err = Error::from_msg_srcs(
                format!(
                    "Unknown {} name `{}`.",
                    accept_type_string,
                    short_name.to_string()
                ),
                &[&src],
            );
            err.code = Some(ERR_UNKNOWN_NAME);
            err.data = Some(serde_json::Value::String(short_name.to_string()));
            Err(Errors::from_err(err))
        } else if candidates.len() == 1 {
            Ok(candidates[0].clone())
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
