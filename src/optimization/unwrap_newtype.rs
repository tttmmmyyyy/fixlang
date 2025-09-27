// Unwrap newtype pattern, i.e., type A = unbox struct { data : B } to B.

use crate::ast::program::{Program, Symbol};

pub fn run(prg: &mut Program) {
    for (_name, sym) in &mut prg.symbols {
        run_on_symbol(sym);
    }
}

fn run_on_symbol(_sym: &mut Symbol) {
    // if let Some(body) = &mut sym.body {
    //     let new_body = body.unwrap_newtype(&prg.env).unwrap();
    //     *body = new_body;
    // }
}
