//! The ABI choices that keep a Fix function's tail calls compiled as jumps.
//!
//! A monadic loop is a chain of indirect tail calls — `Std::IO`'s `bind` ends with
//! `(f(a).@runner)(iostate)` — so it runs in constant stack only while the backend turns those calls
//! into jumps. Two independent properties of the signature decide whether it does.
//!
//! **The return value must fit in the return registers.** A value that does not fit is returned
//! through a pointer, which LLVM introduces on its own at instruction selection, and
//! `X86TargetLowering::IsEligibleForTailCallOptimization` then rejects a caller or callee that
//! returns that way. Emitting the pointer as an ordinary parameter instead keeps the jumps: the tail
//! call forwards the caller's own out-pointer, and the backend produces `jmp` (direct) or
//! `jmpq *%rdx` (indirect). `lambda_function_type` calls `returns_through_out_pointer` to decide
//! which functions get one.
//!
//! The predicate LLVM itself uses is `TargetLowering::CanLowerReturn`, which neither the C API nor
//! inkwell exposes, so the budget lives here as a table keyed by target architecture. Adding a
//! target, or raising the LLVM version, means revisiting the table; the constant-stack tests in
//! `test_wide_return_tail_call.rs` are what make a stale entry visible.
//!
//! **The arguments whose values change must fit in the argument registers.** Beyond them arguments
//! travel on the stack, and an x86-64 sibcall may only reuse a stack slot that already holds the
//! value being passed — six changing integer arguments is the limit there. Fix passes an unbox
//! struct as its leaf scalars, so a loop carrying its state in arguments reaches that quickly: an
//! out-pointer, a four-leaf state and a capture pointer already fill it. `LAMBDA_CALLING_CONVENTION`
//! lifts this limit for every Fix lambda.

use inkwell::types::BasicTypeEnum;

/// `tailcc`, the convention Fix lambdas are defined and called with. It lets the backend rewrite the
/// stack arguments of a tail call rather than requiring them to already hold the values being
/// passed, so a tail call carrying more arguments than the argument registers hold still becomes a
/// jump. It leaves the return-register budget alone, so a wide result still needs its out-pointer.
///
/// Every Fix lambda is defined with it (`declare_lambda_function`, `declare_rc_function`) and called
/// with it (`apply_lambda`), and nothing else uses it: `main`, the exported wrappers, the runtime,
/// the FFI declarations, the traversers, the reference-counting helpers and the global accessors all
/// keep the C convention. A pointer type carries no convention, so a definition and a call that
/// disagree corrupt silently instead of failing to verify.
///
/// The value is `llvm::CallingConv::Tail`, which inkwell exposes only as a number.
pub const LAMBDA_CALLING_CONVENTION: u32 = 18;

/// How many registers of each class a target returns a value in.
#[derive(Clone, Copy)]
pub struct ReturnRegisters {
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

/// The budget of the target a module is built for. `Generator` reads it once from the module's
/// triple, which `create_module` copies from the target machine, so it follows the target rather
/// than the host.
///
/// An architecture outside the table gets `X86_64`, the smallest entry, so an unlisted target
/// returns through the out-pointer wherever a listed one might have used registers. The cost of that
/// is optimization headroom, while the cost of guessing a budget too large is O(n) stack with
/// nothing to signal it.
pub fn return_registers_of_target(triple: &str) -> ReturnRegisters {
    match triple.split('-').next().unwrap_or("") {
        "x86_64" => X86_64,
        "aarch64" | "arm64" => AARCH64,
        _ => X86_64,
    }
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
/// This must depend on the leaf types and the target alone. Under separated compilation the units
/// that define a function and that call it are generated apart, so a decision reading anything else
/// could differ between the two and break the ABI between them.
pub fn returns_through_out_pointer(leaf_tys: &[BasicTypeEnum], budget: ReturnRegisters) -> bool {
    let demand = leaf_tys
        .iter()
        .map(|ty| demand_of(*ty))
        .fold(RegisterDemand::default(), RegisterDemand::plus);
    demand.unmodeled > 0 || demand.integer > budget.integer || demand.float > budget.float
}
