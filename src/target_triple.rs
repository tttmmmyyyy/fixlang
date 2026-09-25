//! The parts of an LLVM target triple, `<architecture>-<vendor>-<operating system>[-<environment>]`,
//! that decide how the compiler meets a target's ABI.
//!
//! A triple is read after LLVM normalizes it. The default triple of an LLVM build is the one it was
//! configured with, as given, and a triple in the GNU form such as `aarch64-linux-gnu` leaves the
//! vendor out. Normalization puts each part at its position, filling a missing one with `unknown`.

use inkwell::targets::{TargetMachine, TargetTriple};

/// The architectures whose ABI the compiler knows.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Architecture {
    /// x86-64, which LLVM names `x86_64`.
    X86_64,
    /// 64-bit ARM, which LLVM names `aarch64` or `arm64`.
    AArch64,
    /// Any other architecture.
    Other,
}

/// The architecture a target triple names.
///
/// # Examples
/// `x86_64-unknown-linux-gnu` names `X86_64`, and both `aarch64-unknown-linux-gnu` and
/// `arm64-apple-darwin23.0.0` name `AArch64`.
pub fn architecture_of_target(triple: &str) -> Architecture {
    match normalized_parts(triple)[0].as_str() {
        "x86_64" => Architecture::X86_64,
        "aarch64" | "arm64" => Architecture::AArch64,
        _ => Architecture::Other,
    }
}

/// The vendor a target triple names, and `unknown` for a triple that names none.
///
/// # Examples
/// `arm64-apple-darwin23.0.0` names `apple`, and both `aarch64-unknown-linux-gnu` and
/// `aarch64-linux-gnu` name `unknown`.
pub fn vendor_of_target(triple: &str) -> String {
    normalized_parts(triple).swap_remove(1)
}

/// The parts of a target triple, after LLVM normalizes it, in the order
/// `<architecture>-<vendor>-<operating system>[-<environment>]`.
fn normalized_parts(triple: &str) -> Vec<String> {
    let normalized = TargetMachine::normalize_triple(&TargetTriple::create(triple));
    normalized
        .as_str()
        .to_string_lossy()
        .split('-')
        .map(str::to_string)
        .collect()
}
