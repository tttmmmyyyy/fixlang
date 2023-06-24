use super::*;

#[derive(Clone)]
pub struct ImportStatement {
    pub source_module: Name,
    pub target_module: Name,
    pub source: Option<Span>,
}
