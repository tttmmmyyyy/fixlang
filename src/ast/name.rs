use crate::constants::{MODULE_SEPARATOR, NAMESPACE_SEPARATOR, PATTERN_WILDCARD_VAR_PREFIX};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fmt::{self, Debug, Formatter};
use std::hash::{Hash, Hasher};

/// The name of a module, a namespace, a type, a trait, a value or a field, as it is written in
/// source.
pub type Name = String;

/// The path of names an entity is written under: the `Std::Iterator` of `Std::Iterator::empty`.
// PROOF: P2a, P15, P16, P17, P18 (dev-docs/proof/rc_ir/borrow-cancel)
#[derive(Clone, Serialize, Deserialize)]
pub struct NameSpace {
    /// The names of the path, the outermost first.
    pub names: Vec<String>,
    /// Whether the path was written with a leading `::`, as `::Main::x` is and `Main::x` is not.
    /// A leading `::` says that the first name is a module's, so it cannot be a namespace's.
    pub is_absolute: bool,
}

impl Hash for NameSpace {
    /// Hashes the names of the path; `is_absolute` takes no part, as it takes none in equality.
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.names.hash(state);
    }
}

impl PartialEq for NameSpace {
    /// Whether the two paths hold the same names; `is_absolute` takes no part.
    fn eq(&self, other: &Self) -> bool {
        self.names == other.names
    }
}

impl Eq for NameSpace {}

impl PartialOrd for NameSpace {
    /// Orders by the rendered path, such as `Std::Iterator`.
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.to_string().cmp(&other.to_string()))
    }
}

impl Ord for NameSpace {
    /// Orders by the rendered path, such as `Std::Iterator`.
    fn cmp(&self, other: &Self) -> Ordering {
        self.to_string().cmp(&other.to_string())
    }
}

impl NameSpace {
    /// The empty path, which a name written with no qualification carries.
    pub fn local() -> Self {
        Self {
            names: vec![],
            is_absolute: false,
        }
    }

    /// A path of `names`, the outermost first, written with no leading `::`.
    pub fn new(names: Vec<String>) -> Self {
        Self {
            names,
            is_absolute: false,
        }
    }

    /// Marks the path as one written with a leading `::`.
    pub fn set_absolute(&mut self) {
        self.is_absolute = true;
    }

    /// A path of `names`, the outermost first, written with no leading `::`.
    pub fn from_strs(names: &[&str]) -> Self {
        Self::new(names.iter().map(|s| s.to_string()).collect())
    }

    /// Whether the path holds no name, as the path of a name written with no qualification does.
    pub fn is_local(&self) -> bool {
        self.names.len() == 0
    }

    /// Reads the last name of the path as an entity's name and the names before it as the namespace
    /// it lies under. The path must hold at least one name.
    ///
    /// # Examples
    /// `NameSpace::from_strs(&["Std", "Iterator"]).to_fullname()` is the name `Iterator` under the
    /// namespace `Std`.
    pub fn to_fullname(mut self) -> FullName {
        assert!(!self.names.is_empty());
        let name = self.names.pop().unwrap();
        FullName {
            namespace: self,
            name,
        }
    }

    /// The names joined by `::`. An absolute path renders as a relative one does.
    ///
    /// # Examples
    /// `NameSpace::from_strs(&["Std", "Iterator"]).to_string()` is `"Std::Iterator"`.
    // PROOF: P27, P29, P30 (dev-docs/proof/rc_ir/borrow-cancel)
    pub fn to_string(&self) -> String {
        self.names.join(NAMESPACE_SEPARATOR)
    }

    /// Whether `self` names the tail of `rhs`, down to the parts a module name is built from: a
    /// module name is split at `.` as the path is at `::`. An absolute `self` matches `rhs` whole.
    ///
    /// # Examples
    /// `NameSpace::from_strs(&["Name"])` is a suffix of `NameSpace::from_strs(&["Mod.Name"])`, and
    /// is no suffix of `NameSpace::from_strs(&["ModName"])`.
    pub fn is_suffix_of(&self, rhs: &NameSpace) -> bool {
        /// Splits the path at `::` and each name at `.`, so `Mod.Name::entity` becomes
        /// `[Mod, Name, entity]`.
        fn to_components(namespace: &NameSpace) -> Vec<String> {
            if namespace.names.is_empty() {
                return vec![];
            }
            let text = namespace.to_string();
            let text = text.replace(NAMESPACE_SEPARATOR, MODULE_SEPARATOR);
            text.split(MODULE_SEPARATOR)
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

    /// Whether the names of `self` head the names of `rhs`.
    ///
    /// # Examples
    /// `NameSpace::from_strs(&["Std"])` is a prefix of
    /// `NameSpace::from_strs(&["Std", "Iterator"])`.
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

    /// The number of names in the path.
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.names.len()
    }

    /// The module the path lies in, which is the first of its names. The path must hold at least
    /// one name.
    pub fn module(&self) -> Name {
        self.names[0].clone()
    }

    /// The names of `self` followed by the names of `rhs`, as a path written with no leading `::`.
    pub fn append(&self, mut rhs: NameSpace) -> NameSpace {
        let mut names = self.names.clone();
        names.append(&mut rhs.names);
        NameSpace::new(names)
    }

    /// Removes the first name of the path, and reports whether there was one to remove.
    pub fn pop_front(&mut self) -> bool {
        if self.names.is_empty() {
            return false;
        }
        self.names.remove(0);
        true
    }

    /// Puts `name` before the names of the path.
    pub fn push_front(&mut self, name: Name) {
        self.names.insert(0, name);
    }

    /// Puts `name` after the names of the path.
    pub fn push_back(&mut self, name: Name) {
        self.names.push(name);
    }

    /// Reads a path from its written form: the names between `::`, with a leading `::` marking the
    /// path absolute.
    ///
    /// # Examples
    /// `NameSpace::parse("Std::Iterator")` holds the names `["Std", "Iterator"]`, and
    /// `NameSpace::parse("::Std::Iterator")` holds the same names marked absolute.
    pub fn parse(s: &str) -> Option<Self> {
        if s.is_empty() {
            return None;
        }
        let mut is_absolute = false;
        let mut names = s
            .split(NAMESPACE_SEPARATOR)
            .map(|name| name.to_owned())
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

/// An entity's name together with the path it is written under: `Std::Iterator::empty` is the name
/// `empty` under the namespace `Std::Iterator`.
// PROOF: P2a, P8, P9, P10, P11, P12, P13, P14, P14a, P14b, P15, P16, P17, P18 (dev-docs/proof/rc_ir/borrow-cancel)
#[derive(Eq, PartialEq, Clone, Serialize, Deserialize)]
pub struct FullName {
    /// The path the entity is written under.
    pub namespace: NameSpace,
    /// The name of the entity itself.
    pub name: String,
}

impl Hash for FullName {
    /// Hashes the names of the namespace and the entity's name; `is_absolute` takes no part.
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.namespace.names.hash(state);
        self.name.hash(state);
    }
}

impl Debug for FullName {
    /// Writes the full name as it is written in source, with the leading `::` of an absolute name.
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}{}",
            if self.is_absolute() { "::" } else { "" },
            self.to_string()
        )
    }
}

impl PartialOrd for FullName {
    /// Orders by the rendered full name, such as `Std::Iterator::empty`; `is_absolute` takes no
    /// part.
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.to_string().cmp(&other.to_string()))
    }
}

impl Ord for FullName {
    /// Orders by the rendered full name, such as `Std::Iterator::empty`; `is_absolute` takes no
    /// part.
    fn cmp(&self, other: &Self) -> Ordering {
        self.to_string().cmp(&other.to_string())
    }
}

impl FullName {
    /// The name `name` under the namespace `ns`.
    pub fn new(ns: &NameSpace, name: &str) -> Self {
        Self {
            namespace: ns.clone(),
            name: name.to_string(),
        }
    }

    /// The name `name` under the namespace `ns` spells out, the outermost name first.
    pub fn from_strs(ns: &[&str], name: &str) -> Self {
        Self::new(&NameSpace::from_strs(ns), name)
    }

    /// The name `name` written with no namespace.
    // PROOF: (P-insert) (dev-docs/proof/rc_ir/borrow-cancel)
    pub fn local(name: &str) -> Self {
        Self::new(&NameSpace::local(), name)
    }

    /// Whether the name is written with no namespace, as a local variable's is.
    // PROOF: P27, P29, P30, (P-insert) (dev-docs/proof/rc_ir/borrow-cancel)
    pub fn is_local(&self) -> bool {
        return self.namespace.is_local();
    }

    /// Whether the name is written under a namespace, as a global value's is.
    pub fn is_global(&self) -> bool {
        return !self.is_local();
    }

    /// Whether this is the binder the compiler makes for a `_` the user wrote in a pattern. Such a
    /// binder is displayed as `_`, and source refers to it nowhere.
    pub fn is_pattern_wildcard(&self) -> bool {
        self.name.starts_with(PATTERN_WILDCARD_VAR_PREFIX)
    }

    /// The name as it is shown to the user, which for a pattern-wildcard binder is the `_` the user
    /// wrote.
    pub fn display_name(&self) -> String {
        if self.is_pattern_wildcard() {
            "_".to_string()
        } else {
            self.to_string()
        }
    }

    /// Reads `self` as a path relative to `container`, and returns it with `container` before its
    /// namespace. A `self` written with a leading `::` is returned as it stands, keeping the
    /// `is_absolute` flag that says so.
    ///
    /// # Examples
    /// `FullName::local("x").join_under(&NameSpace::from_strs(&["Main"]))` is `Main::x`, and a
    /// `self` written `::Lib::x` stays `::Lib::x` under any `container`.
    pub fn join_under(mut self, container: &NameSpace) -> FullName {
        if self.namespace.is_absolute {
            return self;
        }
        let mut joined = container.names.clone();
        joined.append(&mut self.namespace.names);
        self.namespace.names = joined;
        self.namespace.is_absolute = false;
        self
    }

    /// The namespace and the name joined by `::`.
    ///
    /// # Examples
    /// `FullName::from_strs(&["Std", "Iterator"], "empty").to_string()` is
    /// `"Std::Iterator::empty"`.
    // PROOF: P27, P29, P30 (dev-docs/proof/rc_ir/borrow-cancel)
    pub fn to_string(&self) -> String {
        let ns = self.namespace.to_string();
        if ns.is_empty() {
            self.name.clone()
        } else {
            ns + NAMESPACE_SEPARATOR + &self.name
        }
    }

    /// Whether `self` names the tail of `other`: the same entity name, under a namespace `self`'s
    /// namespace is a suffix of.
    pub fn is_suffix_of(&self, other: &FullName) -> bool {
        self.name == other.name && self.namespace.is_suffix_of(&other.namespace)
    }

    /// The whole name read as a path, with the entity's name as its last name.
    // PROOF: P27, P29, P30 (dev-docs/proof/rc_ir/borrow-cancel)
    pub fn to_namespace(&self) -> NameSpace {
        let mut names = self.namespace.names.clone();
        names.push(self.name.clone());
        NameSpace {
            names,
            is_absolute: self.namespace.is_absolute,
        }
    }

    /// The module the name lies in, which is the first name of its namespace.
    // PROOF: P27, P29, P30 (dev-docs/proof/rc_ir/borrow-cancel)
    pub fn module(&self) -> Name {
        self.namespace.module()
    }

    /// A mutable handle on the entity's name, leaving the namespace as it is.
    pub fn name_as_mut(&mut self) -> &mut Name {
        &mut self.name
    }

    /// Removes the first name of the namespace, and reports whether there was one to remove.
    pub fn pop_front_namespace(&mut self) -> bool {
        self.namespace.pop_front()
    }

    /// Puts `name` before the names of the namespace.
    pub fn push_front(&mut self, name: Name) {
        self.namespace.push_front(name);
    }

    /// Whether `namespace` heads this name's namespace, so the name lies within it.
    ///
    /// # Examples
    /// `FullName::from_strs(&["Std", "Iterator"], "empty")` is in the namespace
    /// `NameSpace::from_strs(&["Std"])`.
    pub fn is_in_namespace(&self, namespace: &NameSpace) -> bool {
        namespace.is_prefix_of(&self.namespace)
    }

    /// Reads a full name from its written form: the last name between `::` as the entity's, the
    /// names before it as its namespace, and a leading `::` marking the path absolute.
    ///
    /// # Examples
    /// `FullName::parse("Std::Iterator::empty")` is the name `empty` under `Std::Iterator`.
    pub fn parse(s: &str) -> Option<FullName> {
        if s.is_empty() {
            return None;
        }
        let mut namespace = NameSpace::parse(s)?;
        if namespace.names.is_empty() {
            return None;
        }
        let name = namespace.names.pop();
        Some(FullName {
            namespace,
            name: name.unwrap(),
        })
    }

    /// Whether the name was written with a leading `::`.
    pub fn is_absolute(&self) -> bool {
        self.namespace.is_absolute
    }

    /// Marks the name as one written with a leading `::`.
    pub fn set_absolute(&mut self) {
        self.namespace.is_absolute = true;
    }

    /// Marks a name written under a namespace as absolute, and leaves a name written with none as
    /// it stands.
    pub fn global_to_absolute(&mut self) {
        if !self.is_local() {
            self.namespace.is_absolute = true;
        }
    }
}
