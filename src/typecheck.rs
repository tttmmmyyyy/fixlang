use super::*;

#[derive(Debug)]
pub struct TypeError {}

pub fn check_type(ei: Arc<ExprInfo>, /* inference context */) -> Result<Arc<ExprInfo>, TypeError> {
    todo!("check typcon arity")
}
