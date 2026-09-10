//! Building a small RC IR program in a test, out of the names its bodies mention.
//!
//! A pass that reads which name reaches which — dead-code elimination, the division into
//! compilation units — is checked by handing it a program whose content is those mentions and
//! nothing else, so a function or a global here is built from the list of names its body mentions.

use crate::ast::name::FullName;
use crate::ast::types::type_funptr;
use crate::fixstd::builtin::make_i64_ty;
use crate::misc::Set;
use crate::rc_ir::ast::{
    FuncRef, RcExpr, RcExprNode, RcFunc, RcGlobalInit, RcProgram, RcRhs, RcVar,
};
use std::sync::Arc;

/// The name lowering gives a symbol of the program under test.
pub fn global_name(name: &str) -> FullName {
    FullName::from_strs(&["Main"], name)
}

/// A variable of type `I64` under `name`, carrying no source location or debug name.
pub fn var(name: FullName) -> RcVar {
    RcVar {
        name,
        ty: make_i64_ty(),
        source: None,
        debug_name: None,
        skip_null_check: false,
    }
}

/// A body that mentions each of `mentions` — as the reference of a closure value — and returns the
/// last value it bound. A body mentioning nothing returns its own parameter.
fn body_mentioning(mentions: &[FullName]) -> RcExprNode {
    let last = FullName::local(&format!("v{}", mentions.len()));
    let mut body = RcExprNode {
        expr: Arc::new(RcExpr::Ret(var(last))),
        source: None,
    };
    for (i, mentioned) in mentions.iter().enumerate().rev() {
        body = RcExprNode {
            expr: Arc::new(RcExpr::Let(
                var(FullName::local(&format!("v{}", i + 1))),
                RcRhs::Closure(
                    FuncRef {
                        name: mentioned.clone(),
                    },
                    vec![],
                ),
                body,
            )),
            source: None,
        };
    }
    body
}

/// A function of one `I64` parameter whose body mentions each of `mentions`.
pub fn func(name: FullName, mentions: &[FullName]) -> RcFunc {
    RcFunc {
        name: FuncRef { name },
        fn_ty: type_funptr(vec![make_i64_ty()], make_i64_ty()),
        params: vec![var(FullName::local("v0"))],
        capture: None,
        ret_ty: make_i64_ty(),
        body: body_mentioning(mentions),
        source: None,
        borrowed_units: Set::default(),
        inline_into_callers: false,
    }
}

/// A global value whose initializer mentions each of `mentions`.
pub fn global(symbol: FullName, mentions: &[FullName]) -> RcGlobalInit {
    RcGlobalInit {
        symbol,
        ty: make_i64_ty(),
        init: body_mentioning(mentions),
        owns_initializer: true,
        owns_storage: true,
    }
}

/// A program of `funcs` and `globals` reached through `roots`.
pub fn prog(funcs: Vec<RcFunc>, globals: Vec<RcGlobalInit>, roots: &[FullName]) -> RcProgram {
    RcProgram {
        funcs: funcs.into_iter().map(|f| (f.name.clone(), f)).collect(),
        globals,
        roots: roots.iter().cloned().collect(),
    }
}
