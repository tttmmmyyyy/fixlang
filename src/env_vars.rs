use crate::configuration::FixOptimizationLevel;
use crate::misc::warn_msg;
use std::env;

/// The environment variable holding the highest optimization level a build works at. It also
/// decides the level of a build that asks for none.
pub const MAX_OPT_LEVEL_VAR: &str = "FIX_MAX_OPT_LEVEL";

/// The highest optimization level a build works at, as `MAX_OPT_LEVEL_VAR` gives it. A value the
/// variable does not hold, and a value it holds that names no level, both give `Max`.
pub fn get_max_opt_level() -> FixOptimizationLevel {
    if let Ok(var) = env::var(MAX_OPT_LEVEL_VAR) {
        if let Some(level) = FixOptimizationLevel::from_str(&var) {
            return level;
        }
        warn_msg(&format!(
            "Invalid value for {}: \"{}\". Using default value.",
            MAX_OPT_LEVEL_VAR, var
        ));
    }
    FixOptimizationLevel::Max
}
