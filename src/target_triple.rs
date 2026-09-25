//! The parts of an LLVM target triple, `<architecture>-<vendor>-<operating system>[-<environment>]`,
//! that decide how the compiler meets a target's ABI.

/// The architectures whose ABI the compiler knows.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Architecture {
    /// x86-64.
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
    match triple.split('-').next().unwrap() {
        "x86_64" => Architecture::X86_64,
        "aarch64" | "arm64" => Architecture::AArch64,
        _ => Architecture::Other,
    }
}

/// The vendor a target triple names, and the empty string for a triple that names none.
///
/// # Examples
/// `arm64-apple-darwin23.0.0` names `apple`, and `aarch64-unknown-linux-gnu` names `unknown`.
pub fn vendor_of_target(triple: &str) -> &str {
    triple.split('-').nth(1).unwrap_or("")
}
