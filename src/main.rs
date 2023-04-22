extern crate pest;
#[macro_use]
extern crate pest_derive;
#[allow(unused_imports)]
#[macro_use]
extern crate serial_test;
// extern crate rustc_llvm_proxy;

mod ast;
mod builtin;
mod c_config;
mod constants;
mod funptr_optimization;
mod generator;
mod llvm_passes;
mod misc;
mod object;
mod parser;
mod runner;
mod runtime;
mod stdlib;
#[cfg(test)]
mod tests;
mod typecheck;

use ast::expr::*;
use ast::import::*;
use ast::module::*;
use ast::traits::*;
use ast::typedecl::*;
use ast::types::*;
use builtin::*;
use c_config::*;
use clap::{App, AppSettings, Arg};
use constants::*;
use funptr_optimization::*;
use generator::*;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::passes::PassManager;
use inkwell::support::load_library_permanently;
use inkwell::types::{BasicTypeEnum, FunctionType, IntType, PointerType, StructType};
use inkwell::values::{
    BasicValue, BasicValueEnum, CallableValue, FunctionValue, IntValue, PointerValue,
};
use inkwell::{AddressSpace, IntPredicate, OptimizationLevel};
use llvm_passes::*;
use misc::*;
use object::*;
use parser::*;
use pest::iterators::{Pair, Pairs};
use pest::Parser;
use runner::*;
use runtime::*;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::rc::Rc;
use std::vec::Vec;
use stdlib::*;
use typecheck::*;

// Max number of arguments if function pointer lambda.
pub const FUNPTR_ARGS_MAX: u32 = 100;
// Max tuple size.
// This affects on compilation time heavily. We should make tuple generation on-demand in a future.
pub const TUPLE_SIZE_MAX: u32 = 4;
// Is tuple unboxed?
pub const TUPLE_UNBOX: bool = true;

#[derive(Clone)]
pub struct Configuration {
    // Runs memory sanitizer to detect memory leak and invalid memory reference at early time.
    // Requires shared library sanitizer/libfixsanitizer.so.
    sanitize_memory: bool,
    // Perform function pointer optimization.
    funptr_optimization: bool,
    // If true, pre-retain global object (i.e., set refcnt to large value) at it's construction
    // and do not retain global object thereafter.
    preretain_global: bool,
    // LLVM optimization level.
    llvm_opt_level: OptimizationLevel,
}

impl Configuration {
    // Configuration for release build.
    pub fn release() -> Configuration {
        Configuration {
            sanitize_memory: false,
            funptr_optimization: true,
            preretain_global: true,
            llvm_opt_level: OptimizationLevel::Default,
        }
    }

    // Usual configuration for compiler development
    pub fn develop_compiler() -> Configuration {
        Configuration {
            sanitize_memory: true,
            funptr_optimization: true,
            preretain_global: false,
            llvm_opt_level: OptimizationLevel::Default,
        }
    }
}

fn main() {
    let source_file = Arg::new("source-file").required(true);
    let run_subc = App::new("run").arg(source_file.clone());
    let build_subc = App::new("build").arg(source_file.clone());
    let app = App::new("Fix-lang")
        .bin_name("fix")
        .setting(AppSettings::ArgRequiredElseHelp)
        .subcommand(run_subc)
        .subcommand(build_subc);

    match app.get_matches().subcommand() {
        Some(("run", m)) => {
            let path = m.value_of("source-file").unwrap();
            run_file(Path::new(path), Configuration::release());
        }
        Some(("build", m)) => {
            let path = m.value_of("source-file").unwrap();
            build_file(Path::new(path), Configuration::release());
        }
        _ => eprintln!("Unknown command!"),
    }
}
