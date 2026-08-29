//! The C side of the FFI boundary: the type of a C function, and how a Fix value crosses to it.
//!
//! A program describes a C function in two ways — an `FFI_CALL` declares one it calls, and an
//! `FFI_EXPORT` defines one it offers — and the compiler describes the entry point it writes. All
//! three reach the module as one function, so `CSignature` is what they have to agree on and
//! `CSignature::get_or_declare_in_module` is where they meet.

use crate::ast::export_statement::ExportedFunctionType;
use crate::ast::name::{FullName, Name};
use crate::ast::program::TypeEnv;
use crate::ast::types::{tycon, TyCon, TypeNode};
use crate::constants::{
    F32_NAME, F64_NAME, I16_NAME, I32_NAME, I64_NAME, I8_NAME, PTR_NAME, STD_NAME, U16_NAME,
    U32_NAME, U64_NAME, U8_NAME,
};
use crate::generator::Generator;
use inkwell::attributes::AttributeLoc;
use inkwell::context::Context;
use inkwell::types::{BasicMetadataTypeEnum, BasicType, BasicTypeEnum};
use inkwell::values::{BasicValueEnum, FunctionValue};
use inkwell::AddressSpace;
use std::sync::Arc;

/// How C carries a value: everything the declaration of a C function exchanging it says about it.
///
/// Two types of one shape are one C type, so a signature written with either declares the same
/// function. `I64` and `U64` share a shape: a value that fills its register travels the same way
/// whichever sign the reader gives the bits. `I8` and `U8` do not, since the ABI carries a narrow
/// integer in the low bits of a register and the sign is what says which side extends it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CTypeShape {
    /// An integer of this width in bits, carrying the extension its width earns it.
    Integer {
        /// The width of the C integer type: 8, 16, 32 or 64.
        bits: u32,
        /// The extension the ABI asks of a value of this width, and `None` at a width that fills
        /// the unit a C signature carries an integer in.
        extension: Option<CIntegerExtension>,
    },
    /// C's `float`.
    Float32,
    /// C's `double`.
    Float64,
    /// A pointer, which C carries as one address whatever it points at.
    Pointer,
}

/// How the bits above an integer narrower than the 32-bit unit the ABI carries it in are filled.
///
/// Apple's AArch64 has the caller fill them for an argument and the callee for a result, and lets
/// the other side read the whole register on that promise, while AAPCS64 and System V leave them
/// unspecified and have the reader narrow the value itself. Naming the fill is how a signature says
/// which of the two it follows, and a C compiler puts one on every such parameter and result.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CIntegerExtension {
    /// Copies of the value's sign bit fill the bits above it.
    Sign,
    /// Zeroes fill the bits above the value.
    Zero,
}

impl CIntegerExtension {
    /// The name of the LLVM attribute carrying this extension.
    pub fn attribute_name(self) -> &'static str {
        match self {
            CIntegerExtension::Sign => "signext",
            CIntegerExtension::Zero => "zeroext",
        }
    }
}

/// The unit a C signature carries an integer in. An integer narrower than this width travels in the
/// low bits of the unit, and C's default argument promotions widen such an integer to this width on
/// the way through `...`.
///
/// The width holds for the targets Fix builds for; an ABI that extends a 32-bit integer to the width
/// of a register — RISC-V 64 does — raises it.
const C_INTEGER_UNIT_BITS: u32 = 32;

/// How a Fix type constructor crosses to C.
impl TyCon {
    /// The shape of the C type this type constructor stands for.
    /// `()` is C's `void`, which carries no value, so it has no shape.
    ///
    /// An integer narrower than 32 bits travels in the low bits of a register and carries the
    /// extension its sign asks for; one that fills the register carries none, which is what lets a
    /// program read the same C function's result as `I64` in one place and as `U64` in another.
    pub fn c_type_shape(self: &TyCon) -> Option<CTypeShape> {
        if self.is_unit() {
            return None;
        }
        assert!(
            self.is_c_scalar(),
            "call c_type_shape for {}",
            self.to_string()
        );
        let integer = |bits| CTypeShape::Integer {
            bits,
            extension: if bits < C_INTEGER_UNIT_BITS {
                Some(if self.is_signed_integer() {
                    CIntegerExtension::Sign
                } else {
                    CIntegerExtension::Zero
                })
            } else {
                None
            },
        };
        Some(match self.name.name.as_str() {
            I8_NAME | U8_NAME => integer(8),
            I16_NAME | U16_NAME => integer(16),
            I32_NAME | U32_NAME => integer(32),
            I64_NAME | U64_NAME => integer(64),
            F32_NAME => CTypeShape::Float32,
            F64_NAME => CTypeShape::Float64,
            PTR_NAME => CTypeShape::Pointer,
            // `C_SCALAR_NAMES` gained a name that this mapping does not cover.
            name => unreachable!("no C type for `{}`", name),
        })
    }

    /// The extension the ABI puts on a value of this type crossing to C, and `None` for a value that
    /// needs none: a wide integer, a floating point number, a pointer, and `()`.
    pub fn c_integer_extension(self: &TyCon) -> Option<CIntegerExtension> {
        match self.c_type_shape()? {
            CTypeShape::Integer { extension, .. } => extension,
            _ => None,
        }
    }

    /// How a C declaration spells the type this type constructor stands for.
    ///
    /// An integer is written in the fixed-width name `<stdint.h>` gives it, since the width is what
    /// the Fix type fixes and the spelling C would otherwise use — `int`, `long` — is a different
    /// width on a different target.
    ///
    /// # Examples
    /// `I32` is written `int32_t`, `U8` is `uint8_t`, `Ptr` is `void *`, and `()` is `void`.
    pub fn c_type_name(self: &TyCon) -> String {
        if self.is_unit() {
            return "void".to_string();
        }
        assert!(
            self.is_c_scalar(),
            "call c_type_name for {}",
            self.to_string()
        );
        match self.name.name.as_str() {
            I8_NAME => "int8_t",
            U8_NAME => "uint8_t",
            I16_NAME => "int16_t",
            U16_NAME => "uint16_t",
            I32_NAME => "int32_t",
            U32_NAME => "uint32_t",
            I64_NAME => "int64_t",
            U64_NAME => "uint64_t",
            F32_NAME => "float",
            F64_NAME => "double",
            PTR_NAME => "void *",
            // `C_SCALAR_NAMES` gained a name that this mapping does not cover.
            name => unreachable!("no C spelling for `{}`", name),
        }
        .to_string()
    }

    /// Convert `()`, `I8`, `Ptr`, etc. to the corresponding C type.
    /// `()` is C's `void`, which carries no value, so it maps to `None`.
    pub fn get_c_type<'c>(self: &TyCon, ctx: &'c Context) -> Option<BasicTypeEnum<'c>> {
        Some(match self.c_type_shape()? {
            CTypeShape::Integer { bits, .. } => {
                ctx.custom_width_int_type(bits).as_basic_type_enum()
            }
            CTypeShape::Float32 => ctx.f32_type().as_basic_type_enum(),
            CTypeShape::Float64 => ctx.f64_type().as_basic_type_enum(),
            CTypeShape::Pointer => ctx.ptr_type(AddressSpace::from(0)).as_basic_type_enum(),
        })
    }
}

/// The type of a C function, as the Fix type constructors standing for the C types it exchanges.
///
/// A program describes a C function in two ways: an `FFI_CALL` declares one it calls, and an
/// `FFI_EXPORT` defines one it offers. Both descriptions of one name reach the module as a single
/// function, through which every call in the program goes, so they have to give one signature.
pub struct CSignature {
    /// The type of each parameter, in the order the C function takes them.
    pub param_tys: Vec<Arc<TyCon>>,
    /// The type of the result, the unit type where the C function returns nothing.
    pub ret_tycon: Arc<TyCon>,
    /// Whether the signature ends in `...`.
    pub is_var_args: bool,
}

/// The signature of the entry point the compiler generates: `int32_t (int32_t, void *)`, taking
/// `argc` and `argv` as the C runtime passes them.
///
/// The compiler writes this function's body, so it is the one C function a program describes without
/// naming it. A program that does name it — an `FFI_CALL` of `main`, which re-runs the program —
/// gives this signature, and `Program::validate_c_function_calls` reports one that gives another.
pub fn c_entry_point_signature() -> CSignature {
    let std_tycon = |name: &str| tycon(FullName::from_strs(&[STD_NAME], name));
    CSignature {
        param_tys: vec![std_tycon(I32_NAME), std_tycon(PTR_NAME)],
        ret_tycon: std_tycon(I32_NAME),
        is_var_args: false,
    }
}

impl CSignature {
    /// The signature an `FFI_CALL` writes for the function it calls.
    pub fn of_ffi_call(
        ret_tycon: &Arc<TyCon>,
        param_tys: &Vec<Arc<TyCon>>,
        is_var_args: bool,
    ) -> CSignature {
        CSignature {
            param_tys: param_tys.clone(),
            ret_tycon: ret_tycon.clone(),
            is_var_args,
        }
    }

    /// The signature of the C function generated for a value exported at `exported_ty`, whose every
    /// position `ExportedFunctionType::validate` has admitted as one the C ABI can carry.
    pub fn of_ffi_export(exported_ty: &ExportedFunctionType, type_env: &TypeEnv) -> CSignature {
        let boundary_tycon = |ty: &Arc<TypeNode>| {
            c_boundary_tycon(ty, type_env)
                .unwrap_or_else(|| panic!("`{}` reached an exported signature", ty.to_string()))
        };
        CSignature {
            param_tys: exported_ty.doms.iter().map(boundary_tycon).collect(),
            ret_tycon: if exported_ty.codom.is_unit() {
                exported_ty.codom.toplevel_tycon().unwrap()
            } else {
                boundary_tycon(&exported_ty.codom)
            },
            is_var_args: false,
        }
    }

    /// Whether this signature and `other` declare the same C function: the two describe every
    /// position the same way, down to what a declaration of it carries — `CTypeShape` holds which
    /// differences between two Fix types that is, and which it is not.
    pub fn agrees_with(&self, other: &CSignature) -> bool {
        self.is_var_args == other.is_var_args
            && self.param_tys.len() == other.param_tys.len()
            && self.ret_tycon.c_type_shape() == other.ret_tycon.c_type_shape()
            && (self.param_tys.iter())
                .zip(other.param_tys.iter())
                .all(|(a, b)| a.c_type_shape() == b.c_type_shape())
    }

    /// The function `name` of this signature in the module, declaring it where nothing declares it
    /// yet. Every description of one C name goes through here, which is what puts the calls a
    /// program makes and the definition it exports on one function.
    pub fn get_or_declare_in_module<'c, 'm>(
        &self,
        name: &Name,
        gc: &Generator<'c, 'm>,
    ) -> FunctionValue<'c> {
        if let Some(declared) = gc.module.get_function(name) {
            return declared;
        }
        let param_c_tys = self
            .param_tys
            .iter()
            .map(|param_ty| {
                // A parameter of type `()` is rejected where the signature is written: `void` is a
                // result alone.
                param_ty.get_c_type(gc.context).unwrap().into()
            })
            .collect::<Vec<BasicMetadataTypeEnum>>();
        let fn_ty = match self.ret_tycon.get_c_type(gc.context) {
            None => gc
                .context
                .void_type()
                .fn_type(&param_c_tys, self.is_var_args),
            Some(ret_c_ty) => ret_c_ty.fn_type(&param_c_tys, self.is_var_args),
        };
        let func = gc.module.add_function(name, fn_ty, None);
        assert_eq!(
            func.get_name().to_str().unwrap(),
            name,
            "the C function enters the module under the name it was given"
        );
        gc.add_c_integer_extension_attribute(func, AttributeLoc::Return, &self.ret_tycon);
        for (i, param_ty) in self.param_tys.iter().enumerate() {
            gc.add_c_integer_extension_attribute(func, AttributeLoc::Param(i as u32), param_ty);
        }
        func
    }

    /// The signature as a C declaration of the function `name` reads.
    ///
    /// # Examples
    /// A function of two arguments reads `int32_t c_pick(int32_t, int32_t)`, one of none
    /// `void c_now(void)`, and a variadic one `int32_t c_report(void *, ...)`.
    pub fn declaration_of(&self, name: &Name) -> String {
        let mut params = self
            .param_tys
            .iter()
            .map(|param_ty| param_ty.c_type_name())
            .collect::<Vec<_>>();
        if self.is_var_args {
            params.push("...".to_string());
        }
        // C writes the empty parameter list of a declaration as `(void)`.
        if params.is_empty() {
            params.push("void".to_string());
        }
        format!(
            "{} {}({})",
            self.ret_tycon.c_type_name(),
            name,
            params.join(", ")
        )
    }
}

/// The C type a value of `ty` crosses the FFI boundary as, and `None` for a value the C ABI cannot
/// carry the way Fix lays it down.
///
/// A value with one scalar — an integer, a floating point number, or a pointer — is laid down
/// identically by Fix and by C, and a boxed value is a pointer. An aggregate is laid down
/// differently: the C ABI classifies a structure by its size and by the class of each of its
/// eightbytes (System V AMD64), or by whether it is a homogeneous floating-point aggregate
/// (AAPCS64), and the shapes on which that agrees with Fix's element-wise layout differ from target
/// to target.
pub fn c_boundary_tycon(ty: &Arc<TypeNode>, type_env: &TypeEnv) -> Option<Arc<TyCon>> {
    let head = ty.toplevel_tycon()?;
    if ty.is_box(type_env) {
        return Some(tycon(FullName::from_strs(&[STD_NAME], PTR_NAME)));
    }
    if !head.is_c_scalar() {
        return None;
    }
    Some(head)
}

/// The message reporting that a value of `ty` cannot be passed through the `...` of an `FFI_CALL`,
/// and `None` for a type that can.
///
/// A call hands a variadic argument to C as the one scalar the value is, and C's default argument
/// promotions are stated over C types, so the argument has to be a value C carries as one scalar. A
/// declared parameter is written as such a type; past the `...` the type comes from inference alone,
/// which is why this checks the type the argument was inferred to.
///
/// A boxed value is admitted at an exported signature, where it crosses as an opaque pointer, and
/// refused here, where the call would hand C the first word of the heap block instead of the address
/// of it.
pub fn unpassable_variadic_type_msg(ty: &Arc<TypeNode>) -> Option<String> {
    if ty.toplevel_tycon().map_or(false, |head| head.is_c_scalar()) {
        return None;
    }
    let msg_head = format!(
        "`{}` cannot be passed through the `...` of an `FFI_CALL`",
        ty.to_string()
    );
    if ty.is_boolean() {
        return Some(msg_head + ". Use `U8` or `CInt`, and convert it on the Fix side.");
    }
    if ty.is_string() {
        return Some(
            msg_head
                + ". Use `Std::String::borrow_c_str` to get a `Ptr` to its bytes, and pass that.",
        );
    }
    Some(msg_head + ". An argument passing through `...` is an integer (`I8` to `I64`, `U8` to `U64`), a floating point number (`F32`, `F64`), or a pointer (`Ptr`). The C types in `Std::FFI` such as `CInt` are aliases of these. To pass a boxed value, take a `Ptr` to it with `Std::FFI::boxed_to_retained_ptr` or `borrow_boxed`.")
}

/// Widen `val`, the value of Fix type `ty` a call has marshalled, the way C widens an argument going
/// through the `...`.
///
/// C's default argument promotions turn a `float` into a `double`, and widen an integer narrower
/// than `C_INTEGER_UNIT_BITS` to that width, filling the bits above the value the way its sign asks.
/// This is why a C function reads its variadic arguments as `double` and `int`: a narrower value
/// never arrives, so a call has to write the value the function reads. A value that already fills
/// that width is handed over as it stands.
pub fn promote_through_ellipsis<'c, 'm>(
    val: BasicValueEnum<'c>,
    ty: &Arc<TypeNode>,
    gc: &Generator<'c, 'm>,
) -> BasicValueEnum<'c> {
    let head = ty
        .toplevel_tycon()
        .filter(|head| head.is_c_scalar())
        .unwrap_or_else(|| {
            panic!(
                "`{}` reached a variadic argument, which `Program::validate_c_function_calls` \
                 admits only as a C scalar",
                ty.to_string()
            )
        });
    match head
        .c_type_shape()
        .expect("a C scalar is carried by a C type")
    {
        CTypeShape::Integer {
            extension: Some(extension),
            ..
        } => {
            let unit_ty = gc.context.custom_width_int_type(C_INTEGER_UNIT_BITS);
            let val = val.into_int_value();
            let builder = gc.builder();
            match extension {
                CIntegerExtension::Sign => builder.build_int_s_extend(val, unit_ty, "promoted"),
                CIntegerExtension::Zero => builder.build_int_z_extend(val, unit_ty, "promoted"),
            }
            .unwrap()
            .into()
        }
        CTypeShape::Float32 => gc
            .builder()
            .build_float_ext(val.into_float_value(), gc.context.f64_type(), "promoted")
            .unwrap()
            .into(),
        CTypeShape::Integer {
            extension: None, ..
        }
        | CTypeShape::Float64
        | CTypeShape::Pointer => val,
    }
}

/// Assert that a value of Fix type `ty` travels as the one scalar the C type `c_ty` names, which is
/// what lets the generated function hand it to C as it stands.
///
/// Counting the parts alone would let an aggregate through, since a value too wide to split is
/// carried as one part holding the whole of it, and C would then be handed a structure whose layout
/// it classifies by its own rules. `c_boundary_tycon` admits nothing with either shape.
pub fn assert_crosses_as_c_type<'c, 'm>(
    c_ty: &Arc<TyCon>,
    ty: &Arc<TypeNode>,
    gc: &mut Generator<'c, 'm>,
) {
    let embedded_ty = ty.get_embedded_type(gc);
    let parts = gc.type_parts(embedded_ty);
    assert_eq!(
        parts.len(),
        1,
        "`{}` reached an exported signature, where a value has to be one scalar",
        ty.to_string()
    );
    assert_eq!(
        parts[0],
        c_ty.get_c_type(gc.context).unwrap(),
        "`{}` crosses as the C type `{}`, so the two lay it down alike",
        ty.to_string(),
        c_ty.to_string()
    );
}
