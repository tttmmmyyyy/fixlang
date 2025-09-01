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
            items: vec![ImportTreeNode::from_names(names)],
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
}

#[derive(Clone)]
pub enum ImportTreeNode {
    Any(Option<Span>),
    Symbol(Name, Option<Span>),
    TypeOrTrait(Name, Option<Span>),
    NameSpace(Name, Vec<ImportTreeNode>, Option<Span>),
}

impl ImportTreeNode {
    // From a list of names, for example ["A", "B", "f"], create `Namespace("A", [Namespace("B", [Symbol("f")])])`.
    fn from_names(mut names: Vec<Name>) -> ImportTreeNode {
        if names.len() == 0 {
            return ImportTreeNode::Any(None);
        }
        if names.len() == 1 {
            let name = names.pop().unwrap();
            // If the first letter of `name` is lowercase, create a symbol node.
            if name.chars().next().unwrap().is_lowercase() {
                return ImportTreeNode::Symbol(name, None);
            }
            // If the first letter of `name` is uppercase, create a type or trait node.
            return ImportTreeNode::TypeOrTrait(name, None);
        }
        let next_names = names.split_off(1);
        let namespace = names.pop().unwrap();
        ImportTreeNode::NameSpace(
            namespace,
            vec![ImportTreeNode::from_names(next_names)],
            None,
        )
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
