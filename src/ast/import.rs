use name::{FullName, Name, NameSpace};

use super::*;

pub fn is_accessible(stmts: &[ImportStatement], name: &FullName) -> bool {
    stmts.iter().any(|stmt| stmt.is_accessible(name))
}

#[derive(Clone)]
pub struct ImportStatement {
    pub importer: Name,
    pub module: (Name, Option<Span>),
    pub items: Vec<ImportTreeNode>,
    pub hiding: Vec<ImportTreeNode>,
    pub source: Option<Span>,
    // Is this import statement is added implicitly by compiler?
    // The module itself and `Std` module are imported implicitly.
    pub implicit: bool,
}

impl ImportStatement {
    pub fn sort(stmts: &mut [ImportStatement]) {
        stmts.sort_by(|a, b| a.module.0.cmp(&b.module.0));
        for stmt in stmts {
            ImportTreeNode::sort(&mut stmt.items);
            ImportTreeNode::sort(&mut stmt.hiding);
        }
    }

    pub fn find_node_at(&self, pos: &SourcePos) -> Option<EndNode> {
        let span = self.module.1.as_ref()?;
        if !span.includes_pos(pos) {
            return None;
        }
        return Some(EndNode::Module(self.module.0.clone()));
    }

    // Checks if the given name is made accessible by this import statement.
    pub fn is_accessible(&self, name: &FullName) -> bool {
        if name.module() != self.module.0 {
            return false;
        }
        let mut name = name.clone();
        assert!(name.pop_front_namespace());
        if self
            .hiding
            .iter()
            .any(|hidden_item| hidden_item.is_accessible(&name))
        {
            return false;
        }
        self.items.iter().any(|item| item.is_accessible(&name))
    }

    pub fn implicit_self_import(module: Name) -> ImportStatement {
        ImportStatement {
            importer: module.clone(),
            module: (module, None),
            items: vec![ImportTreeNode::Any(None)],
            hiding: vec![],
            source: None,
            implicit: true,
        }
    }

    pub fn implicit_std_import(module: Name) -> ImportStatement {
        ImportStatement {
            importer: module,
            module: (STD_NAME.to_string(), None),
            items: vec![ImportTreeNode::Any(None)],
            hiding: vec![],
            source: None,
            implicit: true,
        }
    }

    pub fn import_to_use(importer: Name, name: FullName) -> ImportStatement {
        let module = name.module();
        let mut names = name.to_namespace().names.clone();
        let names = names.split_off(1);
        ImportStatement {
            importer,
            module: (module, None),
            items: vec![ImportTreeNode::from_names(&names)],
            hiding: vec![],
            source: None,
            implicit: false,
        }
    }

    // Returns the items that are referred by this import statement.
    // Includes items that are hidden.
    pub fn referred_items(&self) -> Vec<ImportItem> {
        let mut result = vec![];
        for item in &self.items {
            result.append(&mut ImportTreeNode::items(item));
        }
        for item in &self.hiding {
            result.append(&mut ImportTreeNode::items(item));
        }
        for item in &mut result {
            item.push_front(self.module.0.clone());
        }
        result
    }

    pub fn stringify(&self) -> String {
        let mut res = format!("import {}", self.module.0);
        if self.items.len() == 0 {
            res += "::{}"
        }
        if self.items.len() == 1 && matches!(self.items[0], ImportTreeNode::Any(_)) {
            // "import Std::*" can be written as "import Std"
        } else {
            if self.items.len() >= 1 {
                res += "::";
                if self.items.len() >= 2 {
                    res += "{";
                }
                res += self
                    .items
                    .iter()
                    .map(|item| item.stringify())
                    .collect::<Vec<_>>()
                    .join(", ")
                    .as_str();
                if self.items.len() >= 2 {
                    res += "}";
                }
            }
        }
        if self.hiding.len() >= 1 {
            if self.hiding.len() >= 2 {
                res += " hiding {";
            } else {
                res += " hiding ";
            }
            res += self
                .hiding
                .iter()
                .map(|item| item.stringify())
                .collect::<Vec<_>>()
                .join(", ")
                .as_str();
            if self.hiding.len() >= 2 {
                res += "}";
            }
        }
        res += ";";
        res
    }

    // Adds a new import statement for the given name.
    pub fn add_import(imports: &mut Vec<ImportStatement>, importer: Name, name: FullName) {
        // If it's already accessible, do nothing.
        if is_accessible(&imports, &name) {
            return;
        }
        // Find an import statement which has the same module name to `name` no hiding items.
        let module = name.module();
        let import = imports
            .iter()
            .enumerate()
            .find(|(_i, import)| import.module.0 == module && import.hiding.is_empty());
        // If there is no such import, create a new one.
        if import.is_none() {
            let new_import = ImportStatement::import_to_use(importer, name);
            imports.push(new_import);
            return;
        }
        // If found, add the name to the import statement.
        let idx = import.unwrap().0;
        let import = &mut imports[idx];
        import.add_item(&name.to_namespace().names[1..]);
    }

    fn add_item(&mut self, names: &[Name]) {
        assert!(names.len() >= 1);
        assert!(self.hiding.is_empty());
        ImportTreeNode::add_item_internal(&mut self.items, names);
    }
}

#[derive(Clone)]
pub enum ImportTreeNode {
    Any(Option<Span>),
    Symbol(Name, Option<Span>),
    TypeOrTrait(Name, Option<Span>),
    NameSpace(Name, Vec<ImportTreeNode>, Option<Span>),
}

impl ImportTreeNode {
    fn sort(nodes: &mut [ImportTreeNode]) {
        nodes.sort_by(|a, b| {
            // Any < Symbol (cmp by name) < TypeOrTrait (cmp by name) < Namespace (cmp by name)
            match (a, b) {
                (ImportTreeNode::Any(_), ImportTreeNode::Any(_)) => std::cmp::Ordering::Equal,
                (ImportTreeNode::Any(_), _) => std::cmp::Ordering::Less,
                (_, ImportTreeNode::Any(_)) => std::cmp::Ordering::Greater,
                (ImportTreeNode::Symbol(name_a, _), ImportTreeNode::Symbol(name_b, _)) => {
                    name_a.cmp(name_b)
                }
                (ImportTreeNode::Symbol(_, _), _) => std::cmp::Ordering::Less,
                (_, ImportTreeNode::Symbol(_, _)) => std::cmp::Ordering::Greater,
                (
                    ImportTreeNode::TypeOrTrait(name_a, _),
                    ImportTreeNode::TypeOrTrait(name_b, _),
                ) => name_a.cmp(name_b),
                (ImportTreeNode::TypeOrTrait(_, _), _) => std::cmp::Ordering::Less,
                (_, ImportTreeNode::TypeOrTrait(_, _)) => std::cmp::Ordering::Less,
                (
                    ImportTreeNode::NameSpace(name_a, _, _),
                    ImportTreeNode::NameSpace(name_b, _, _),
                ) => name_a.cmp(name_b),
            }
        });
        for node in nodes {
            if let ImportTreeNode::NameSpace(_, items, _) = node {
                Self::sort(items);
            }
        }
    }

    // From a list of names, for example ["A", "B", "f"], create `Namespace("A", [Namespace("B", [Symbol("f")])])`.
    fn from_names(names: &[Name]) -> ImportTreeNode {
        if names.len() == 0 {
            return ImportTreeNode::Any(None);
        }
        if names.len() == 1 {
            let name = &names[0];
            // If the first letter of `name` is lowercase, create a symbol node.
            if name.chars().next().unwrap().is_lowercase() {
                return ImportTreeNode::Symbol(name.clone(), None);
            }
            // If the first letter of `name` is uppercase, create a type or trait node.
            return ImportTreeNode::TypeOrTrait(name.clone(), None);
        }
        let namespace = &names[0];
        ImportTreeNode::NameSpace(
            namespace.clone(),
            vec![ImportTreeNode::from_names(&names[1..])],
            None,
        )
    }

    fn add_item(&mut self, names: &[Name]) {
        assert!(names.len() >= 1);
        assert!(matches!(self, ImportTreeNode::NameSpace(_, _, _)));
        if let ImportTreeNode::NameSpace(_, items, _) = self {
            ImportTreeNode::add_item_internal(items, names);
            return;
        } else {
            unreachable!()
        }
    }

    fn add_item_internal(items: &mut Vec<ImportTreeNode>, names: &[Name]) {
        assert!(names.len() >= 1);
        if names.len() >= 2 {
            let namespace = &names[0];
            // If `items` already includes the `namespace`, then add the `names` to it.
            if let Some(item) = items.iter_mut().find(|item| match item {
                ImportTreeNode::NameSpace(name, _, _) => name == namespace,
                _ => false,
            }) {
                item.add_item(&names[1..]);
                return;
            }
            // If `items` does not include the `namespace`, then add the `names`.
            items.push(ImportTreeNode::from_names(names));
            return;
        }
        // `names` has no namespace.
        let name = &names[0];
        // If `name` is already included in `items`, do nothing.
        if items.iter().any(|item| match item {
            ImportTreeNode::Symbol(symbol, _) => symbol == name,
            ImportTreeNode::TypeOrTrait(symbol, _) => symbol == name,
            _ => false,
        }) {
            return;
        }
        // If `name` is not already included in `items`, then add it.
        items.push(ImportTreeNode::from_names(names));
    }

    pub fn is_accessible(&self, name: &FullName) -> bool {
        match self {
            ImportTreeNode::Any(_) => true,
            ImportTreeNode::Symbol(symbol, _) => name.is_local() && name.name == *symbol,
            ImportTreeNode::TypeOrTrait(symbol, _) => name.is_local() && name.name == *symbol,
            ImportTreeNode::NameSpace(symbol, filters, _) => {
                if name.is_local() {
                    return false;
                }
                if name.namespace.names[0] != *symbol {
                    return false;
                }
                let mut name = name.clone();
                assert!(name.pop_front_namespace());
                filters.iter().any(|filter| filter.is_accessible(&name))
            }
        }
    }

    fn items(item: &ImportTreeNode) -> Vec<ImportItem> {
        match item {
            ImportTreeNode::Any(src) => {
                vec![ImportItem::NameSpace(NameSpace::new(vec![]), src.clone())]
            }
            ImportTreeNode::Symbol(name, src) => {
                vec![ImportItem::Symbol(FullName::local(name), src.clone())]
            }
            ImportTreeNode::TypeOrTrait(name, src) => {
                vec![ImportItem::TypeOrTrait(FullName::local(name), src.clone())]
            }
            ImportTreeNode::NameSpace(name, items, _src) => {
                let mut result = vec![];
                for item in items {
                    let mut childs = Self::items(item);
                    for child in &mut childs {
                        child.push_front(name.clone());
                    }
                    result.append(&mut childs);
                }
                result
            }
        }
    }

    fn stringify(&self) -> String {
        match self {
            ImportTreeNode::Any(_) => "*".to_string(),
            ImportTreeNode::Symbol(name, _) => name.clone(),
            ImportTreeNode::TypeOrTrait(name, _) => name.clone(),
            ImportTreeNode::NameSpace(name, items, _) => {
                let mut res = name.clone();
                if items.len() >= 1 {
                    if items.len() >= 2 {
                        res += "::{";
                    } else {
                        res += "::";
                    }
                    res += &items
                        .iter()
                        .map(|item| item.stringify())
                        .collect::<Vec<_>>()
                        .join(", ");
                    if items.len() >= 2 {
                        res += "}";
                    }
                }
                res
            }
        }
    }
}

pub enum ImportItem {
    Symbol(FullName, Option<Span>),
    TypeOrTrait(FullName, Option<Span>),
    NameSpace(NameSpace, Option<Span>),
}

impl ImportItem {
    pub fn push_front(&mut self, name: Name) {
        match self {
            ImportItem::Symbol(fullname, _) => {
                fullname.push_front(name);
            }
            ImportItem::TypeOrTrait(fullname, _) => {
                fullname.push_front(name);
            }
            ImportItem::NameSpace(namespace, _) => {
                namespace.push_front(name);
            }
        }
    }
}
