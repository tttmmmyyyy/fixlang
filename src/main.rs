extern crate pest;
#[macro_use]
extern crate pest_derive;
#[allow(unused_imports)]
#[macro_use]
extern crate serial_test;

mod ast;
mod generator;
mod parser;
mod runner;
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

/*

fn generate_lam<'c, 'm, 'b>(
    arg: Arc<Var>,
    val: Arc<ExprInfo>,
    gc: &mut GenerationContext<'c, 'm, 'b>,
) -> ExprCode<'c> {
    let context = gc.context;
    let module = gc.module;
    // Fix ordering of captured names
    let mut captured_names = val.free_vars.clone();
    captured_names.remove(arg.name());
    captured_names.remove(SELF_NAME);
    let captured_names: Vec<String> = captured_names.into_iter().collect();
    // Determine the type of closure
    let mut field_types = vec![
        ObjectFieldType::ControlBlock,
        ObjectFieldType::LambdaFunction,
    ];
    for _ in captured_names.iter() {
        field_types.push(ObjectFieldType::SubObject);
    }
    let obj_type = ObjectType { field_types };
    let closure_ty = obj_type.to_struct_type(context);
    // Declare lambda function
    let lam_fn_ty = lambda_function_type(context);
    let lam_fn = module.add_function("lambda", lam_fn_ty, None);
    // Implement lambda function
    {
        // Create new builder
        let builder = gc.context.create_builder();
        let bb = context.append_basic_block(lam_fn, "entry");
        builder.position_at_end(bb);
        // Create new scope
        let mut scope = LocalVariables::default();
        let arg_ptr = lam_fn.get_first_param().unwrap().into_pointer_value();
        scope.push(&arg.name(), &ExprCode { ptr: arg_ptr });
        let closure_obj = lam_fn.get_nth_param(1).unwrap().into_pointer_value();
        scope.push(SELF_NAME, &ExprCode { ptr: closure_obj });
        for (i, cap_name) in captured_names.iter().enumerate() {
            let cap_obj =
                build_get_field(closure_obj, closure_ty, i as u32 + 2, gc).into_pointer_value();
            scope.push(cap_name, &ExprCode { ptr: cap_obj });
        }
        // Create new gc
        let mut gc = GenerationContext {
            context,
            module,
            builder: &builder,
            scope,
            runtimes: gc.runtimes.clone(),
        };
        // Retain captured objects
        for cap_name in &captured_names {
            let ptr = gc.scope.get(cap_name).code.ptr;
            build_retain(ptr, &gc);
        }
        // Release SELF and arg if unused
        if !val.free_vars.contains(SELF_NAME) {
            build_release(closure_obj, &gc);
        }
        if !val.free_vars.contains(arg.name()) {
            build_release(arg_ptr, &gc);
        }
        // Generate value
        let val = generate_expr(val.clone(), &mut gc);
        // Return result
        let ret = builder.build_pointer_cast(val.ptr, ptr_to_object_type(gc.context), "ret");
        builder.build_return(Some(&ret));
    }
    // Allocate and set up closure
    let name = lam(arg, val).expr.to_string();
    let obj = obj_type.build_allocate_shared_obj(gc, Some(name.as_str()));
    build_set_field(obj, 1, lam_fn.as_global_value().as_pointer_value(), gc);
    for (i, cap) in captured_names.iter().enumerate() {
        let ptr = gc.get_var_retained_if_used_later(cap).ptr;
        build_set_field(obj, i as u32 + 2, ptr, gc);
    }
    // Return closure object
    ExprCode { ptr: obj }
}

*/
