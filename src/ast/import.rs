use name::{FullName, Name, NameSpace};

use super::*;

pub fn is_accessible(stmts: &[ImportStatement], name: &FullName) -> bool {
    stmts.iter().any(|stmt| stmt.is_accessible(name))
}

#[derive(Clone)]
pub struct ImportStatement {
    pub importer: Name,
    pub module: Name,
    pub items: Vec<ImportTreeNode>,
    pub hiding: Vec<ImportTreeNode>,
    pub source: Option<Span>,
    pub implicit: bool, // The module itself and `Std` modules is imported implicitly.
}

impl ImportStatement {
    pub fn is_accessible(&self, name: &FullName) -> bool {
        if name.module() != self.module {
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
            module,
            items: vec![ImportTreeNode::Any(None)],
            hiding: vec![],
            source: None,
            implicit: true,
        }
    }

    pub fn implicit_std_import(module: Name) -> ImportStatement {
        ImportStatement {
            importer: module,
            module: STD_NAME.to_string(),
            items: vec![ImportTreeNode::Any(None)],
            hiding: vec![],
            source: None,
            implicit: true,
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
            item.push_front(self.module.clone());
        }
        result
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
