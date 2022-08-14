extern crate pest;
#[macro_use]
extern crate pest_derive;
#[allow(unused_imports)]
#[macro_use]
extern crate serial_test;

mod ast;
mod generator;
mod parser;
mod runtime;
#[cfg(test)]
mod tests;
mod types;

use ast::*;
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

fn execute_main_module<'c>(
    context: &'c Context,
    module: &Module<'c>,
    opt_level: OptimizationLevel,
) -> i64 {
    if SANITIZE_MEMORY {
        assert_eq!(
            load_library_permanently("sanitizer/libfixsanitizer.so"),
            false
        );
    }
    let execution_engine = module.create_jit_execution_engine(opt_level).unwrap();
    unsafe {
        let func = execution_engine
            .get_function::<unsafe extern "C" fn() -> i64>("main")
            .unwrap();
        func.call()
    }
}

fn run_ast(program: Arc<ExprInfo>, opt_level: OptimizationLevel) -> i64 {
    // Add library functions to program.
    let program = let_in(var_var("add"), add(), program);
    let program = let_in(var_var("eq"), eq(), program);
    let program = let_in(var_var("fix"), fix(), program);

    let program = calculate_aux_info(program);

    let context = Context::create();
    let module = context.create_module("main");
    let builder = context.create_builder();
    let mut gc = GenerationContext {
        context: &context,
        module: &module,
        builder: &builder,
        scope: Default::default(),
        system_functions: Default::default(),
    };
    generate_system_functions(&mut gc);

    let main_fn_type = context.i64_type().fn_type(&[], false);
    let main_function = module.add_function("main", main_fn_type, None);

    let entry_bb = context.append_basic_block(main_function, "entry");
    builder.position_at_end(entry_bb);

    let program_result = generate_expr(program, &mut gc);

    let int_obj_ptr = builder.build_pointer_cast(
        program_result.ptr,
        ObjectType::int_obj_type()
            .to_struct_type(&context)
            .ptr_type(AddressSpace::Generic),
        "int_obj_ptr",
    );
    let value = build_get_field(int_obj_ptr, 1, &gc);
    build_release(program_result.ptr, &gc);

    if SANITIZE_MEMORY {
        // Perform leak check
        let check_leak = *gc
            .system_functions
            .get(&SystemFunctions::CheckLeak)
            .unwrap();
        gc.builder.build_call(check_leak, &[], "check_leak");
    }

    if let BasicValueEnum::IntValue(value) = value {
        builder.build_return(Some(&value));
    } else {
        panic!("Given program doesn't return int value!");
    }

    module.print_to_file("ir").unwrap();
    let verify = module.verify();
    if verify.is_err() {
        print!("{}", verify.unwrap_err().to_str().unwrap());
        panic!("LLVM verify failed!");
    }
    execute_main_module(&context, &module, opt_level)
}

fn run_source(source: &str, opt_level: OptimizationLevel) -> i64 {
    let ast = parse_source(source);
    run_ast(ast, opt_level)
}

fn run_file(path: &Path, opt_level: OptimizationLevel) -> i64 {
    let display = path.display();

    let mut file = match File::open(&path) {
        Err(why) => panic!("Couldn't open {}: {}", display, why),
        Ok(file) => file,
    };

    // Read the file contents into a string, returns `io::Result<usize>`
    // ファイルの中身を文字列に読み込む。`io::Result<useize>`を返す。
    let mut s = String::new();
    match file.read_to_string(&mut s) {
        Err(why) => panic!("Couldn't read {}: {}", display, why),
        Ok(_) => (),
    }

    run_source(s.as_str(), opt_level)
}

fn test_run_source(source: &str, answer: i64, opt_level: OptimizationLevel) {
    assert_eq!(run_source(source, opt_level), answer)
}

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
