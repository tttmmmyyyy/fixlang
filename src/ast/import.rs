use super::*;

#[derive(Clone)]
pub struct ImportStatement {
    pub importer: Name,
    pub module: Name,
    pub items: Vec<ImportItem>,
    pub hiding: Vec<ImportItem>,
    pub source: Option<Span>,
    pub implicit: bool, // The module itself and `Std` modules is imported implicitly.
}

#[derive(Clone)]
pub enum ImportItem {
    Any,
    Symbol(Name),
    TypeOrTrait(Name),
    NameSpace(Name, Vec<ImportItem>),
}

pub fn is_accessible(stmts: &[ImportStatement], name: &FullName) -> bool {
    stmts.iter().any(|stmt| stmt.is_accessible(name))
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
            items: vec![ImportItem::Any],
            hiding: vec![],
            source: None,
            implicit: true,
        }
    }

    pub fn implicit_std_import(module: Name) -> ImportStatement {
        ImportStatement {
            importer: module,
            module: STD_NAME.to_string(),
            items: vec![ImportItem::Any],
            hiding: vec![],
            source: None,
            implicit: true,
        }
    }
}

impl ImportItem {
    pub fn is_accessible(&self, name: &FullName) -> bool {
        match self {
            ImportItem::Any => true,
            ImportItem::Symbol(symbol) => name.is_local() && name.name == *symbol,
            ImportItem::TypeOrTrait(symbol) => name.is_local() && name.name == *symbol,
            ImportItem::NameSpace(symbol, filters) => {
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
}
