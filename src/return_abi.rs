//! Whether a function's return value fits in the target's return registers.
//!
//! A value that does not fit is returned through a pointer. LLVM does that on its own, at
//! instruction selection, and the backend then declines to turn the function's tail calls into
//! jumps (`X86TargetLowering::IsEligibleForTailCallOptimization` rejects a caller or callee that
//! returns through a hidden pointer). A monadic loop is a chain of indirect tail calls — `Std::IO`'s
//! `bind` ends with `(f(a).@runner)(iostate)` — so losing those jumps costs it O(n) stack.
//!
//! Emitting the pointer as an ordinary parameter instead keeps the jumps: the tail call forwards the
//! caller's own out-pointer, and the backend turns it into `jmp` (direct) or `jmpq *%rdx`
//! (indirect). `lambda_function_type` uses this module to decide which functions get one.
//!
//! The predicate LLVM itself uses is `TargetLowering::CanLowerReturn`, which neither the C API nor
//! inkwell exposes, so the budget lives here as a table keyed by target architecture. Adding a
//! target, or raising the LLVM version, means revisiting the table; the constant-stack tests in
//! `test_wide_return_tail_call.rs` are what make a stale entry visible.

use std::sync::OnceLock;

use inkwell::{targets::TargetMachine, types::BasicTypeEnum};

/// How many registers of each class a target returns a value in.
#[derive(Clone, Copy)]
struct ReturnRegisters {
    integer: usize,
    float: usize,
}

/// x86-64 returns integers and pointers in RAX, RDX and RCX, and floating-point values in XMM0-XMM3.
/// Linux, macOS and Windows agree.
const X86_64: ReturnRegisters = ReturnRegisters {
    integer: 3,
    float: 4,
};

/// AArch64 returns integers and pointers in X0-X7 and floating-point values in V0-V7. Linux and
/// macOS agree.
const AARCH64: ReturnRegisters = ReturnRegisters {
    integer: 8,
    float: 8,
};

/// The budget of the target being compiled for, which is the host: every module is written by the
/// target machine of `get_target_machine`, which builds for `TargetMachine::get_default_triple`.
///
/// An architecture outside the table gets `X86_64`, the smallest entry, so an unlisted target
/// returns through the out-pointer wherever a listed one might have used registers. The cost of that
/// is optimization headroom, while the cost of guessing a budget too large is O(n) stack with
/// nothing to signal it.
fn host_return_registers() -> ReturnRegisters {
    static REGISTERS: OnceLock<ReturnRegisters> = OnceLock::new();
    *REGISTERS.get_or_init(|| {
        let triple = TargetMachine::get_default_triple();
        let triple = triple.as_str().to_string_lossy().into_owned();
        match triple.split('-').next().unwrap_or("") {
            "x86_64" => X86_64,
            "aarch64" | "arm64" => AARCH64,
            _ => X86_64,
        }
    })
}

/// The registers of each class that returning a value costs.
#[derive(Clone, Copy, Default)]
struct RegisterDemand {
    integer: usize,
    float: usize,
    /// Leaves whose register class this module does not model. Any of them sends the value through
    /// the out-pointer, since the alternative is to guess.
    unmodeled: usize,
}

impl RegisterDemand {
    fn plus(self, other: RegisterDemand) -> RegisterDemand {
        RegisterDemand {
            integer: self.integer + other.integer,
            float: self.float + other.float,
            unmodeled: self.unmodeled + other.unmodeled,
        }
    }

    fn times(self, n: usize) -> RegisterDemand {
        RegisterDemand {
            integer: self.integer * n,
            float: self.float * n,
            unmodeled: self.unmodeled * n,
        }
    }
}

/// What returning `ty` costs. LLVM's return lowering flattens a struct or an array into its scalar
/// elements (`ComputeValueVTs`) and gives each one a register, so this descends through both: a
/// `{ i8, [3 x i64] }` costs four integer registers, one more than x86-64 has.
fn demand_of(ty: BasicTypeEnum) -> RegisterDemand {
    match ty {
        BasicTypeEnum::StructType(st) => (0..st.count_fields())
            .map(|i| demand_of(st.get_field_type_at_index(i).unwrap()))
            .fold(RegisterDemand::default(), RegisterDemand::plus),
        BasicTypeEnum::ArrayType(at) => demand_of(at.get_element_type()).times(at.len() as usize),
        BasicTypeEnum::IntType(_) | BasicTypeEnum::PointerType(_) => RegisterDemand {
            integer: 1,
            ..Default::default()
        },
        BasicTypeEnum::FloatType(_) => RegisterDemand {
            float: 1,
            ..Default::default()
        },
        BasicTypeEnum::VectorType(_) => RegisterDemand {
            unmodeled: 1,
            ..Default::default()
        },
    }
}

/// Whether a function returning these leaf scalars takes an out-pointer for its result and returns
/// `void`. The leaves are a return value in `flatten_to_scalar_leaves` order.
///
/// This must depend on the leaf types alone. Under separated compilation the units that define a
/// function and that call it are generated apart, so a decision reading anything else could differ
/// between the two and break the ABI between them.
pub fn returns_through_out_pointer(leaf_tys: &[BasicTypeEnum]) -> bool {
    let demand = leaf_tys
        .iter()
        .map(|ty| demand_of(*ty))
        .fold(RegisterDemand::default(), RegisterDemand::plus);
    let budget = host_return_registers();
    demand.unmodeled > 0 || demand.integer > budget.integer || demand.float > budget.float
}
