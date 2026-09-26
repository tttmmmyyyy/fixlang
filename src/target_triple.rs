//! The parts of an LLVM target triple, `<architecture>-<vendor>-<operating system>[-<environment>]`,
//! that decide how the compiler meets a target's ABI.
//!
//! A triple is read after LLVM normalizes it. The default triple of an LLVM build is the one it was
//! configured with, as given, and a triple in the GNU form such as `aarch64-linux-gnu` leaves the
//! vendor out. Normalization puts each part at its position, filling a missing one with `unknown`.
//! It keeps each part's spelling, so the names below are the spellings LLVM's `Triple` parses into
//! one architecture or one operating system.

use inkwell::targets::{TargetMachine, TargetTriple};

/// The architectures whose ABI the compiler knows.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Architecture {
    /// x86-64, which LLVM names `x86_64`, `amd64` or `x86_64h`.
    X86_64,
    /// 64-bit ARM, which LLVM names `aarch64`, `arm64`, or a variant of either: `aarch64_lfi`,
    /// `arm64e` or `arm64ec`.
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
    match normalized_part(triple, 0).as_str() {
        "x86_64" | "amd64" | "x86_64h" => Architecture::X86_64,
        "aarch64" | "aarch64_lfi" | "arm64" | "arm64e" | "arm64ec" => Architecture::AArch64,
        _ => Architecture::Other,
    }
}

/// Whether a target triple names one of Apple's operating systems, the ones LLVM calls Darwin.
///
/// # Examples
/// `arm64-apple-darwin23.0.0` and `arm64-apple-ios17.0` do; `aarch64-unknown-linux-gnu` and
/// `aarch64-apple-none-macho` do not.
pub fn target_is_darwin(triple: &str) -> bool {
    let os = normalized_part(triple, 2);
    [
        "darwin",
        "macos",
        "ios",
        "tvos",
        "watchos",
        "driverkit",
        "xros",
        "visionos",
        "bridgeos",
    ]
    .iter()
    .any(|name| os.starts_with(name))
}

/// Whether a target triple names Windows.
///
/// # Examples
/// `x86_64-pc-windows-msvc` and `x86_64-w64-windows-gnu` do; `x86_64-unknown-linux-gnu` does not.
pub fn target_is_windows(triple: &str) -> bool {
    let os = normalized_part(triple, 2);
    os.starts_with("windows") || os.starts_with("win32")
}

/// The part at `index` of a target triple after LLVM normalizes it, and the empty string where the
/// triple has fewer parts, as a triple of an architecture alone does.
fn normalized_part(triple: &str, index: usize) -> String {
    let normalized = TargetMachine::normalize_triple(&TargetTriple::create(triple));
    normalized
        .as_str()
        .to_string_lossy()
        .split('-')
        .nth(index)
        .unwrap_or("")
        .to_string()
}
