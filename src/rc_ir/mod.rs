//! The RC intermediate language (RC IR).
//!
//! RC IR is an A-normal form with a fixed evaluation order, explicit `Retain`/`Release` nodes, and
//! globally unique local names. Lowering translates the AST into it, so that both code generation
//! and reference-counting optimizations read explicit reference-counting operations from a single
//! representation.

pub mod ast;
pub mod borrow;
pub mod codegen;
pub mod lower;
pub mod print;
pub mod provenance;
pub mod rc_insert;
pub mod unique_elim;
