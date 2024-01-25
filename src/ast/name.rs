use serde::{Deserialize, Serialize};

use super::*;

pub type Name = String;

#[derive(Eq, Hash, PartialEq, Clone, Serialize, Deserialize)]
pub struct NameSpace {
    pub names: Vec<String>, // Empty implies it is local.
}

impl NameSpace {
    pub fn local() -> Self {
        Self { names: vec![] }
    }

    pub fn new(names: Vec<String>) -> Self {
        Self { names }
    }

    pub fn new_str(names: &[&str]) -> Self {
        Self::new(names.iter().map(|s| s.to_string()).collect())
    }

    pub fn is_local(&self) -> bool {
        self.names.len() == 0
    }

    pub fn to_string(&self) -> String {
        self.names.join(NAMESPACE_SEPARATOR)
    }

    // Checks if `self` is a suffix of the argument.
    // "Name::entity" is not suffix of "ModName::entity", but should be suffix of "Mod.Name::entity".
    pub fn is_suffix(&self, rhs: &NameSpace) -> bool {
        // Splits `Mod.Name::entity` into `[Mod, Name, entity]`.
        fn to_components(namespace: &NameSpace) -> Vec<String> {
            if namespace.names.is_empty() {
                return vec![];
            }
            let str = namespace.to_string();
            let str = str.replace(NAMESPACE_SEPARATOR, MODULE_SEPARATOR);
            str.split(MODULE_SEPARATOR)
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
        }
        let lhs = to_components(self);
        let rhs = to_components(rhs);
        let n = lhs.len();
        let m = rhs.len();
        if n > m {
            return false;
        }
        for i in 0..n {
            if lhs[n - 1 - i] != rhs[m - i - 1] {
                return false;
            }
        }
        return true;
    }

    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.names.len()
    }

    pub fn module(&self) -> Name {
        self.names[0].clone()
    }

    pub fn append(&self, mut rhs: NameSpace) -> NameSpace {
        let mut names = self.names.clone();
        names.append(&mut rhs.names);
        NameSpace::new(names)
    }
}

#[derive(Eq, Hash, PartialEq, Clone, Serialize, Deserialize)]
pub struct FullName {
    pub namespace: NameSpace,
    pub name: String,
}

impl PartialOrd for FullName {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.to_string().cmp(&other.to_string()))
    }
}

impl Ord for FullName {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.to_string().cmp(&other.to_string())
    }
}

impl FullName {
    pub fn new(ns: &NameSpace, name: &str) -> Self {
        Self {
            namespace: ns.clone(),
            name: name.to_string(),
        }
    }

    pub fn from_strs(ns: &[&str], name: &str) -> Self {
        Self::new(&NameSpace::new_str(ns), name)
    }

    pub fn local(name: &str) -> Self {
        Self::new(&NameSpace::local(), name)
    }

    pub fn is_local(&self) -> bool {
        return self.namespace.is_local();
    }

    pub fn is_global(&self) -> bool {
        return !self.is_local();
    }

    pub fn to_string(&self) -> String {
        let ns = self.namespace.to_string();
        if ns.is_empty() {
            self.name.clone()
        } else {
            ns + NAMESPACE_SEPARATOR + &self.name
        }
    }

    pub fn parse(str: &str) -> Option<Self> {
        let mut names = str
            .split(NAMESPACE_SEPARATOR)
            .map(|s| s.to_owned())
            .collect::<Vec<_>>();
        if names.is_empty() {
            return None;
        }
        let name = names.pop().unwrap();
        if names.len() > 0 {
            Some(FullName::new(&NameSpace { names }, &name))
        } else {
            Some(FullName::local(&name))
        }
    }

    pub fn is_suffix(&self, other: &FullName) -> bool {
        self.name == other.name && self.namespace.is_suffix(&other.namespace)
    }

    pub fn to_namespace(&self) -> NameSpace {
        let mut names = self.namespace.names.clone();
        names.push(self.name.clone());
        NameSpace { names }
    }

    pub fn module(&self) -> Name {
        self.namespace.module()
    }

    pub fn name_as_mut(&mut self) -> &mut Name {
        &mut self.name
    }
}
