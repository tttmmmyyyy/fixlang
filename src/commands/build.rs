use crate::configuration::Configuration;
use crate::error::Errors;

pub fn build(config: &Configuration) -> Result<(), Errors> {
    crate::build::build::build(config)
}
