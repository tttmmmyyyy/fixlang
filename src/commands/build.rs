use crate::error::Errors;
use crate::Configuration;

pub fn build(config: &Configuration) -> Result<(), Errors> {
    crate::build::build::build(config)
}
