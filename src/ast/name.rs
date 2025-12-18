use std::fmt::{Debug, Formatter};

use serde::{Deserialize, Serialize};

use super::*;

pub type Name = String;

#[derive(Clone, Serialize, Deserialize)]
pub struct NameSpace {
    // Items in the namespace.
    pub names: Vec<String>,
    // Is this FullName is given as an absolute path?
    // For example, `Main::x` has a relative namespace, but `::Main::x` has an absolute namespace.
    // The latter expresses that `Main` is a module name and cannot be a namespace.
    pub is_absolute: bool,
}

impl std::hash::Hash for NameSpace {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // Ignore `is_absolute` field.
        self.names.hash(state);
    }
}

impl PartialEq for NameSpace {
    fn eq(&self, other: &Self) -> bool {
        // Ignore `is_absolute` field.
        self.names == other.names
    }
}

impl Eq for NameSpace {}

impl PartialOrd for NameSpace {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.to_string().cmp(&other.to_string()))
    }
}

impl Ord for NameSpace {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.to_string().cmp(&other.to_string())
    }
}

impl NameSpace {
    pub fn local() -> Self {
        Self {
            names: vec![],
            is_absolute: false,
        }
    }

    pub fn new(names: Vec<String>) -> Self {
        Self {
            names,
            is_absolute: false,
        }
    }

    pub fn set_absolute(&mut self) {
        self.is_absolute = true;
    }

    pub fn new_str(names: &[&str]) -> Self {
        Self::new(names.iter().map(|s| s.to_string()).collect())
    }

    pub fn is_local(&self) -> bool {
        self.names.len() == 0
    }

    // Convert to a full name.
    pub fn to_fullname(mut self) -> FullName {
        assert!(!self.names.is_empty());
        let name = self.names.pop().unwrap();
        FullName {
            namespace: self,
            name,
        }
    }

    pub fn to_string(&self) -> String {
        self.names.join(NAMESPACE_SEPARATOR)
    }

    // Checks if `self` is a suffix of the argument.
    // "Name::entity" is not suffix of "ModName::entity", but should be suffix of "Mod.Name::entity".
    pub fn is_suffix_of(&self, rhs: &NameSpace) -> bool {
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
        if self.is_absolute {
            // If `lhs` is absolute, then `lhs` is a suffix of `rhs` iff components of `lhs` and `rhs` are completely same.
            return lhs == rhs;
        }
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

    pub fn is_prefix_of(&self, rhs: &NameSpace) -> bool {
        let n = self.names.len();
        let m = rhs.names.len();
        if n > m {
            return false;
        }
        for i in 0..n {
            if self.names[i] != rhs.names[i] {
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

    pub fn pop_front(&mut self) -> bool {
        if self.names.is_empty() {
            return false;
        }
        self.names.remove(0);
        true
    }

    pub fn push_front(&mut self, name: Name) {
        self.names.insert(0, name);
    }

    pub fn push_baack(&mut self, name: Name) {
        self.names.push(name);
    }

    pub fn parse(str: &str) -> Option<Self> {
        if str.is_empty() {
            return None;
        }
        let mut is_absolute = false;
        let mut names = str
            .split(NAMESPACE_SEPARATOR)
            .map(|s| s.to_owned())
            .collect::<Vec<_>>();
        if names.is_empty() {
            return None;
        }
        if names[0].is_empty() {
            is_absolute = true;
            names.remove(0);
        }
        if names.iter().any(|s| s.is_empty()) {
            return None;
        }
        Some(NameSpace {
            names: names,
            is_absolute: is_absolute,
        })
    }
}

#[derive(Eq, PartialEq, Clone, Serialize, Deserialize)]
pub struct FullName {
    pub namespace: NameSpace,
    pub name: String,
}

impl std::hash::Hash for FullName {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // Ignore `is_absolute` field in namespace.
        self.namespace.names.hash(state);
        self.name.hash(state);
    }
}

impl Debug for FullName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}",
            if self.is_absolute() { "::" } else { "" },
            self.to_string()
        )
    }
}

impl PartialOrd for FullName {
    // Ignore `is_absolute` field in namespace.
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.to_string().cmp(&other.to_string()))
    }
}

impl Ord for FullName {
    // Ignore `is_absolute` field in namespace.
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

    pub fn is_suffix(&self, other: &FullName) -> bool {
        self.name == other.name && self.namespace.is_suffix_of(&other.namespace)
    }

    pub fn to_namespace(&self) -> NameSpace {
        let mut names = self.namespace.names.clone();
        names.push(self.name.clone());
        NameSpace {
            names,
            is_absolute: self.namespace.is_absolute,
        }
    }

    pub fn module(&self) -> Name {
        self.namespace.module()
    }

    pub fn name_as_mut(&mut self) -> &mut Name {
        &mut self.name
    }

    // Pop the first component.
    // If the namespace is empty, return false.
    pub fn pop_front_namespace(&mut self) -> bool {
        self.namespace.pop_front()
    }

    pub fn push_front(&mut self, name: Name) {
        self.namespace.push_front(name);
    }

    pub fn is_in_namespace(&self, namespace: &NameSpace) -> bool {
        namespace.is_prefix_of(&self.namespace)
    }

    pub fn parse(s: &str) -> Option<FullName> {
        if s.is_empty() {
            return None;
        }
        let mut names = NameSpace::parse(s)?;
        if names.names.is_empty() {
            return None;
        }
        let name = names.names.pop();
        Some(FullName {
            namespace: names,
            name: name.unwrap(),
        })
    }

    pub fn is_absolute(&self) -> bool {
        self.namespace.is_absolute
    }

    pub fn set_absolute(&mut self) {
        self.namespace.is_absolute = true;
    }

    pub fn global_to_absolute(&mut self) {
        if !self.is_local() {
            self.namespace.is_absolute = true;
        }
    }
}
