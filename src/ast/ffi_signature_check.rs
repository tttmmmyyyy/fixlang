//! Reject a program that reaches one C function name through more than one signature.
//!
//! A C symbol has a single signature, so two `FFI_CALL`s to the same name with different ABI
//! signatures cannot both be honored: code generation declares the C function once and every
//! later call reuses that declaration, which the LLVM verifier rejects. Catching the clash here,
//! with the source locations of the conflicting calls, turns that late abort into a clear error.
//!
//! An `FFI_CALL` type name carries a target-dependent width (`CInt` is a 32-bit integer, `CLong`
//! a 64-bit one on a 64-bit target), so signatures are compared at the ABI level: `CInt` and
//! `CUnsignedInt` agree (both a 32-bit integer, indistinguishable to the C ABI), while `CInt` and
//! `CLong` conflict.

use crate::ast::expr::{Expr, ExprNode};
use crate::ast::name::Name;
use crate::ast::traverse::{EndVisitResult, ExprVisitor, StartVisitResult, VisitState};
use crate::ast::types::TyCon;
use crate::configuration::CTypeSizes;
use crate::constants::{
    C_CHAR_NAME, C_DOUBLE_NAME, C_FLOAT_NAME, C_INT_NAME, C_LONG_LONG_NAME, C_LONG_NAME,
    C_SHORT_NAME, C_SIZE_T_NAME, C_UNSIGNED_CHAR_NAME, C_UNSIGNED_INT_NAME,
    C_UNSIGNED_LONG_LONG_NAME, C_UNSIGNED_LONG_NAME, C_UNSIGNED_SHORT_NAME, F32_NAME, F64_NAME,
    I16_NAME, I32_NAME, I64_NAME, I8_NAME, PTR_NAME, U16_NAME, U32_NAME, U64_NAME, U8_NAME,
};
use crate::error::Errors;
use crate::fixstd::builtin::make_tuple_name_abs;
use crate::misc::Map;
use crate::parse::sourcefile::Span;
use std::sync::Arc;

/// The ABI shape of a C type, coarse enough that two Fix types with the same machine
/// representation compare equal — a signed and an unsigned integer of one width, for example.
#[derive(PartialEq, Eq, Clone)]
enum CAbiType {
    /// An integer of the given bit width.
    Int(usize),
    /// A floating-point number of the given bit width.
    Float(usize),
    /// A pointer.
    Pointer,
    /// The `void` return type, written `()` on the Fix side.
    Void,
    /// A type outside the FFI set, keyed by its name so a mismatch still surfaces.
    Other(String),
}

/// Classify a C type by its ABI representation. `c_sizes` supplies the target's widths for the
/// C types whose size is platform-dependent (`CInt`, `CLong`, ...).
fn c_abi_type(tc: &Arc<TyCon>, c_sizes: &CTypeSizes) -> CAbiType {
    if tc.name == make_tuple_name_abs(0) {
        return CAbiType::Void;
    }
    match tc.name.name.as_str() {
        I8_NAME | U8_NAME => CAbiType::Int(8),
        I16_NAME | U16_NAME => CAbiType::Int(16),
        I32_NAME | U32_NAME => CAbiType::Int(32),
        I64_NAME | U64_NAME => CAbiType::Int(64),
        F32_NAME => CAbiType::Float(32),
        F64_NAME => CAbiType::Float(64),
        PTR_NAME => CAbiType::Pointer,
        C_CHAR_NAME | C_UNSIGNED_CHAR_NAME => CAbiType::Int(c_sizes.char),
        C_SHORT_NAME | C_UNSIGNED_SHORT_NAME => CAbiType::Int(c_sizes.short),
        C_INT_NAME | C_UNSIGNED_INT_NAME => CAbiType::Int(c_sizes.int),
        C_LONG_NAME | C_UNSIGNED_LONG_NAME => CAbiType::Int(c_sizes.long),
        C_LONG_LONG_NAME | C_UNSIGNED_LONG_LONG_NAME => CAbiType::Int(c_sizes.long_long),
        C_SIZE_T_NAME => CAbiType::Int(c_sizes.size_t),
        C_FLOAT_NAME => CAbiType::Float(c_sizes.float),
        C_DOUBLE_NAME => CAbiType::Float(c_sizes.double),
        _ => CAbiType::Other(tc.to_string()),
    }
}

/// One `FFI_CALL` site: the C function name it names, the types it calls it at, and the source
/// span of the call for diagnostics.
struct FFICallSite {
    fun_name: Name,
    ret_ty: Arc<TyCon>,
    param_tys: Vec<Arc<TyCon>>,
    is_va_args: bool,
    span: Option<Span>,
}

impl FFICallSite {
    /// The ABI signature two sites are compared by: return type, parameter types, and whether the
    /// C function is variadic.
    fn abi_signature(&self, c_sizes: &CTypeSizes) -> (CAbiType, Vec<CAbiType>, bool) {
        (
            c_abi_type(&self.ret_ty, c_sizes),
            self.param_tys
                .iter()
                .map(|t| c_abi_type(t, c_sizes))
                .collect(),
            self.is_va_args,
        )
    }

    /// Render the signature as it reads inside `FFI_CALL[...]`, e.g. `CInt f(CInt, Ptr)`.
    fn render(&self) -> String {
        let mut params: Vec<String> = self.param_tys.iter().map(|t| t.to_string()).collect();
        if self.is_va_args {
            params.push("...".to_string());
        }
        format!(
            "{} {}({})",
            self.ret_ty.to_string(),
            self.fun_name,
            params.join(", ")
        )
    }
}

/// Gather every `FFI_CALL` site in an expression tree.
struct FFICallCollector {
    sites: Vec<FFICallSite>,
}

/// Collect every `FFI_CALL` site reachable in `expr`, including those nested inside arguments.
fn collect_ffi_call_sites(expr: &Arc<ExprNode>) -> Vec<FFICallSite> {
    let mut collector = FFICallCollector { sites: Vec::new() };
    collector.traverse(expr);
    collector.sites
}

/// Reject any C function name reached through more than one ABI signature across `exprs`.
pub fn check<'a>(
    exprs: impl IntoIterator<Item = &'a Arc<ExprNode>>,
    c_sizes: &CTypeSizes,
) -> Result<(), Errors> {
    let mut by_name: Map<Name, Vec<FFICallSite>> = Map::default();
    for expr in exprs {
        for site in collect_ffi_call_sites(expr) {
            by_name.entry(site.fun_name.clone()).or_default().push(site);
        }
    }

    // Report names in a stable order, and the calls within each name in source order, so the
    // diagnostic does not depend on the hash-map iteration order.
    let mut names: Vec<&Name> = by_name.keys().collect();
    names.sort();

    let mut errors = Errors::empty();
    for name in names {
        let mut sites: Vec<&FFICallSite> = by_name[name].iter().collect();
        sites.sort_by_key(|s| s.span.as_ref().map_or(usize::MAX, |sp| sp.start));

        // Keep the first call of each distinct signature.
        let mut representatives: Vec<&FFICallSite> = Vec::new();
        for site in sites {
            let sig = site.abi_signature(c_sizes);
            if !representatives
                .iter()
                .any(|r| r.abi_signature(c_sizes) == sig)
            {
                representatives.push(site);
            }
        }
        if representatives.len() < 2 {
            continue;
        }

        let rendered = representatives
            .iter()
            .map(|s| format!("`{}`", s.render()))
            .collect::<Vec<_>>()
            .join(" and ");
        let msg = format!(
            "The C function `{}` is called through more than one signature: {}. A C function has a single signature; call it the same way everywhere.",
            name, rendered
        );
        let srcs: Vec<&Option<Span>> = representatives.iter().map(|s| &s.span).collect();
        errors.append(Errors::from_msg_srcs(msg, &srcs));
    }

    if errors.has_error() {
        Err(errors)
    } else {
        Ok(())
    }
}

impl ExprVisitor for FFICallCollector {
    fn start_visit_var(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        StartVisitResult::VisitChildren
    }
    fn end_visit_var(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_llvm(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        StartVisitResult::VisitChildren
    }
    fn end_visit_llvm(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_app(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        StartVisitResult::VisitChildren
    }
    fn end_visit_app(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_lam(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        StartVisitResult::VisitChildren
    }
    fn end_visit_lam(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_let(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        StartVisitResult::VisitChildren
    }
    fn end_visit_let(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_if(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        StartVisitResult::VisitChildren
    }
    fn end_visit_if(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_match(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        StartVisitResult::VisitChildren
    }
    fn end_visit_match(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_tyanno(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        StartVisitResult::VisitChildren
    }
    fn end_visit_tyanno(
        &mut self,
        expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_make_struct(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        StartVisitResult::VisitChildren
    }
    fn end_visit_make_struct(
        &mut self,
        expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_array_lit(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        StartVisitResult::VisitChildren
    }
    fn end_visit_array_lit(
        &mut self,
        expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_ffi_call(
        &mut self,
        expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        if let Expr::FFICall(fun_name, ret_ty, param_tys, is_va_args, _, _) = expr.expr.as_ref() {
            self.sites.push(FFICallSite {
                fun_name: fun_name.clone(),
                ret_ty: ret_ty.clone(),
                param_tys: param_tys.clone(),
                is_va_args: *is_va_args,
                span: expr.source.clone(),
            });
        }
        StartVisitResult::VisitChildren
    }
    fn end_visit_ffi_call(
        &mut self,
        expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_eval(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        StartVisitResult::VisitChildren
    }
    fn end_visit_eval(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        EndVisitResult::unchanged(expr)
    }
}
