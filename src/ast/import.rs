use super::*;

#[derive(Clone)]
pub struct ImportStatement {
    pub module: Name,
    pub source: Option<Span>,
}
