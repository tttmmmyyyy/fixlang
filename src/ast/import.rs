use super::*;

#[derive(Clone)]
pub struct ImportStatement {
    pub importer: Name,
    pub importee: Name,
    pub source: Option<Span>,
}
