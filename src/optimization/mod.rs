mod beta_reduction;
mod dead_symbol_elimination;
mod decapturing;
#[allow(dead_code)]
mod eta_expand;
mod find_usage_of_name;
mod inline;
pub mod optimization;
mod pull_let;
mod remove_hktvs;
mod remove_renaming;
mod remove_tyanno;
mod simplify_symbol_names;
mod uncurry;
mod unique_local_names;
mod unwrap_newtype;
pub mod utils;
