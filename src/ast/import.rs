use super::*;

#[derive(Clone)]
pub struct ImportStatement {
    pub importer: NameSpace,
    pub importee: NameSpace,
    pub source: Option<Span>,
}
