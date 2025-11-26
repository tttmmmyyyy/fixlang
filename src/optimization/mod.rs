mod application_inlining;
mod dead_symbol_elimination;
mod decapturing;
#[allow(dead_code)]
mod eta_expansion;
mod find_usage_of_name;
mod inline;
mod inline_local;
mod let_elimination;
mod optimize_act;
pub mod optimization;
mod pull_let;
mod remove_hktvs;
mod remove_tyanno;
pub mod rename;
mod simplify_symbol_names;
mod uncurry;
mod unique_local_names;
mod unwrap_newtype;
