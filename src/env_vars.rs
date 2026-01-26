use crate::configuration::FixOptimizationLevel;
use crate::misc::warn_msg;
use std::env;

/// Get the maximum optimization level from the FIX_MAX_OPT_LEVEL environment variable.
/// Returns None if the environment variable is not set or has an invalid value.
pub fn get_max_opt_level() -> FixOptimizationLevel {
    if let Ok(var) = env::var("FIX_MAX_OPT_LEVEL") {
        if let Some(level) = FixOptimizationLevel::from_str(&var) {
            return level;
        }
        warn_msg(&format!(
            "Invalid value for FIX_MAX_OPT_LEVEL: \"{}\". Using default value.",
            var
        ));
    }
    FixOptimizationLevel::Max
}
