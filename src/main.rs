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
mod parser;
mod runner;
mod runtime;
#[cfg(test)]
mod tests;
mod types;

use ast::*;
use builtin::*;
use clap::{App, AppSettings, Arg};
use either::Either;
use generator::*;
use inkwell::basic_block::BasicBlock;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::support::load_library_permanently;
use inkwell::types::{BasicTypeEnum, FunctionType, IntType, PointerType, StructType};
use inkwell::values::{
    BasicValue, BasicValueEnum, CallableValue, FunctionValue, IntValue, PointerValue,
};
use inkwell::{AddressSpace, IntPredicate, OptimizationLevel};
use once_cell::sync::Lazy;
use parser::*;
use pest::iterators::{Pair, Pairs};
use pest::Parser;
use runner::*;
use runtime::*;
use std::alloc::System;
use std::collections::{HashMap, HashSet};
use std::ffi::CString;
use std::fmt::Pointer;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::ptr::null;
use std::string;
use std::sync::Arc;
use std::thread::panicking;
use std::vec::Vec;
use types::*;
use Either::Right;

const SANITIZE_MEMORY: bool = true;

fn main() {
    let source_file = Arg::new("source-file").required(true);
    let run_subcom = App::new("run").arg(source_file);
    let app = App::new("Fix-lang")
        .bin_name("fix")
        .setting(AppSettings::ArgRequiredElseHelp)
        .subcommand(run_subcom);

    match app.get_matches().subcommand() {
        Some(("run", m)) => {
            let path = m.value_of("source-file").unwrap();
            let res = run_file(Path::new(path), OptimizationLevel::Default);
            println!("{}", res);
        }
        _ => eprintln!("Unknown command!"),
    }
}
