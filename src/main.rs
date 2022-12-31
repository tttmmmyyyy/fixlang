extern crate pest;
#[macro_use]
extern crate pest_derive;
#[allow(unused_imports)]
#[macro_use]
extern crate serial_test;
// extern crate rustc_llvm_proxy;

mod ast;
mod builtin;
mod generator;
mod misc;
mod object;
mod parser;
mod runner;
mod runtime;
#[cfg(test)]
mod tests;
mod typecheck;

use ast::expr::*;
use ast::module::*;
use ast::traits::*;
use ast::typedecl::*;
use ast::types::*;
use builtin::*;
use clap::{App, AppSettings, Arg};
use generator::*;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::support::load_library_permanently;
use inkwell::types::{BasicTypeEnum, FunctionType, IntType, PointerType, StructType};
use inkwell::values::{
    BasicValue, BasicValueEnum, CallableValue, FunctionValue, IntValue, PointerValue,
};
use inkwell::{AddressSpace, IntPredicate, OptimizationLevel};
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
use std::sync::Arc;
use std::vec::Vec;
use typecheck::*;

const SANITIZE_MEMORY: bool = false;
const USE_LEAKY_ALLOCATOR: bool = false;
const LEAKY_ALLOCATOR_HEAP_SIZE: u32 = 512 * 1024 * 1024;
const LEAKY_HEAP_NAME: &str = "LEAKY_HEAP";
const TUPLE_SIZE_MAX: u32 = 4; // This affects on compilation time heavily. We should make tuple generation on-demand.

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
            run_file(Path::new(path), OptimizationLevel::Aggressive, false);
        }
        Some(("build", m)) => {
            let path = m.value_of("source-file").unwrap();
            build_file(Path::new(path), OptimizationLevel::Aggressive, false);
        }
        _ => eprintln!("Unknown command!"),
    }
}
