use std::sync::Arc;

use name::FullName;
use serde::{Deserialize, Serialize};

use super::*;

#[derive(Clone, Serialize, Deserialize)]
pub enum LLVMGenerator {
    IntLit(InlineLLVMIntLit),
    FloatLit(InlineLLVMFloatLit),
    NullPtrLit(InlineLLVMNullPtrLit),
    BoolLit(InlineLLVMBoolLit),
    StringBuf(InlineLLVMStringBuf),
    FixBody(InlineLLVMFixBody),
    CastIntegralBody(InlineLLVMCastIntegralBody),
    CastFloatBody(InlineLLVMCastFloatBody),
    CastIntToFloatBody(InlineLLVMCastIntToFloatBody),
    CastFloatToIntBody(InlineLLVMCastFloatToIntBody),
    ShiftBody(InlineLLVMShiftBody),
    BitwiseOperationBody(InlineLLVMBitwiseOperationBody),
    BitNotBody(InlineLLVMBitNotBody),
    FillArrayBody(InlineLLVMFillArrayBody),
    MakeEmptyArrayBody(InlineLLVMMakeEmptyArrayBody),
    ArrayUnsafeSetBoundsUniquenessUncheckedUnreleased(
        InlineLLVMArrayUnsafeSetBoundsUniquenessUncheckedUnreleased,
    ),
    ArrayUnsafeGetBoundsUnchecked(InlineLLVMArrayUnsafeGetBoundsUnchecked),
    ArrayUnsafeSetSizeBody(InlineLLVMArrayUnsafeSetSizeBody),
    ArrayGetBody(InlineLLVMArrayGetBody),
    ArraySetBody(InlineLLVMArraySetBody),
    ArrayForceUniqueBody(InlineLLVMArrayForceUniqueBody),
    ArrayGetPtrBody(InlineLLVMArrayGetPtrBody),
    ArrayGetSizeBody(InlineLLVMArrayGetSizeBody),
    ArrayGetCapacityBody(InlineLLVMArrayGetCapacityBody),
    StructGetBody(InlineLLVMStructGetBody),
    StructSetBody(InlineLLVMStructSetBody),
    StructPunchBody(InlineLLVMStructPunchBody),
    StructPlugInBody(InlineLLVMStructPlugInBody),
    MakeUnionBody(InlineLLVMMakeUnionBody),
    UnionAsBody(InlineLLVMUnionAsBody),
    UnionIsBody(InlineLLVMUnionIsBody),
    UnionModBody(InlineLLVMUnionModBody),
    UndefinedFunctionBody(InlineLLVMUndefinedInternalBody),
    IsUniqueFunctionBody(InlineLLVMIsUniqueFunctionBody),
    IntNegBody(InlineLLVMIntNegBody),
    FloatNegBody(InlineLLVMFloatNegBody),
    BoolNegBody(InlineLLVMBoolNegBody),
    IntEqBody(InlineLLVMIntEqBody),
    PtrEqBody(InlineLLVMPtrEqBody),
    FloatEqBody(InlineLLVMFloatEqBody),
    IntLessThanBody(InlineLLVMIntLessThanBody),
    FloatLessThanBody(InlineLLVMFloatLessThanBody),
    IntLessThanOrEqBody(InlineLLVMIntLessThanOrEqBody),
    FloatLessThanOrEqBody(InlineLLVMFloatLessThanOrEqBody),
    IntAddBody(InlineLLVMIntAddBody),
    FloatAddBody(InlineLLVMFloatAddBody),
    IntSubBody(InlineLLVMIntSubBody),
    FloatSubBody(InlineLLVMFloatSubBody),
    IntMulBody(InlineLLVMIntMulBody),
    FloatMulBody(InlineLLVMFloatMulBody),
    IntDivBody(InlineLLVMIntDivBody),
    FloatDivBody(InlineLLVMFloatDivBody),
    IntRemBody(InlineLLVMIntRemBody),
    MarkThreadedFunctionBody(InlineLLVMMarkThreadedFunctionBody),
    GetRetainedPtrOfBoxedValueFunctionBody(InlineLLVMGetRetainedPtrOfBoxedValueFunctionBody),
    GetBoxedValueFromRetainedPtrFunctionBody(InlineLLVMGetBoxedValueFromRetainedPtrFunctionBody),
    GetReleaseFunctionOfBoxedValueFunctionBody(
        InlineLLVMGetReleaseFunctionOfBoxedValueFunctionBody,
    ),
    GetRetainFunctionOfBoxedValueFunctionBody(InlineLLVMGetRetainFunctionOfBoxedValueFunctionBody),
    GetBoxedDataPtrFunctionBody(InlineLLVMGetBoxedDataPtrFunctionBody),
    WithRetainedFunctionBody(InlineLLVMWithRetainedFunctionBody),
    UnsafeMutateBoxedInternalBody(InlineLLVMUnsafeMutateBoxedInternalFunctionBody),
    UnsafeMutateBoxedIOSInternalBody(InlineLLVMUnsafeMutateBoxedIOSInternalBody),
    ArrayUnsafeGetLinearBoundsUncheckedUnretained(
        InlineLLVMArrayUnsafeGetLinearBoundsUncheckedUnretained,
    ),
    IOStateUnsafeCreate(InlineLLVMIOStateUnsafeCreate),
}

impl LLVMGenerator {
    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        ty: &Arc<TypeNode>,
        tail: bool,
    ) -> Option<Object<'c>> {
        let obj = match self {
            LLVMGenerator::IntLit(x) => Some(x.generate(gc, ty)),
            LLVMGenerator::FloatLit(x) => Some(x.generate(gc, ty)),
            LLVMGenerator::NullPtrLit(x) => Some(x.generate(gc, ty)),
            LLVMGenerator::BoolLit(x) => Some(x.generate(gc, ty)),
            LLVMGenerator::StringBuf(x) => Some(x.generate(gc, ty)),
            LLVMGenerator::FixBody(x) => x.generate(gc, ty, tail),
            LLVMGenerator::CastIntegralBody(x) => Some(x.generate(gc, ty)),
            LLVMGenerator::CastFloatBody(x) => Some(x.generate(gc, ty)),
            LLVMGenerator::CastIntToFloatBody(x) => Some(x.generate(gc, ty)),
            LLVMGenerator::CastFloatToIntBody(x) => Some(x.generate(gc, ty)),
            LLVMGenerator::ShiftBody(x) => Some(x.generate(gc, ty)),
            LLVMGenerator::BitwiseOperationBody(x) => Some(x.generate(gc, ty)),
            LLVMGenerator::BitNotBody(x) => Some(x.generate(gc, ty)),
            LLVMGenerator::FillArrayBody(x) => Some(x.generate(gc, ty)),
            LLVMGenerator::MakeEmptyArrayBody(x) => Some(x.generate(gc, ty)),
            LLVMGenerator::ArrayUnsafeSetBoundsUniquenessUncheckedUnreleased(x) => {
                Some(x.generate(gc, ty))
            }
            LLVMGenerator::ArrayUnsafeGetBoundsUnchecked(x) => Some(x.generate(gc, ty)),
            LLVMGenerator::ArrayUnsafeSetSizeBody(x) => Some(x.generate(gc, ty)),
            LLVMGenerator::ArrayGetBody(x) => Some(x.generate(gc, ty)),
            LLVMGenerator::ArraySetBody(x) => Some(x.generate(gc, ty)),
            LLVMGenerator::ArrayForceUniqueBody(x) => Some(x.generate(gc, ty)),
            LLVMGenerator::ArrayGetPtrBody(x) => Some(x.generate(gc, ty)),
            LLVMGenerator::ArrayGetSizeBody(x) => Some(x.generate(gc, ty)),
            LLVMGenerator::ArrayGetCapacityBody(x) => Some(x.generate(gc, ty)),
            LLVMGenerator::StructGetBody(x) => Some(x.generate(gc, ty)),
            LLVMGenerator::StructSetBody(x) => Some(x.generate(gc, ty)),
            LLVMGenerator::MakeUnionBody(x) => Some(x.generate(gc, ty)),
            LLVMGenerator::UnionAsBody(x) => Some(x.generate(gc, ty)),
            LLVMGenerator::UnionIsBody(x) => Some(x.generate(gc, ty)),
            LLVMGenerator::UnionModBody(x) => Some(x.generate(gc, ty)),
            LLVMGenerator::UndefinedFunctionBody(x) => Some(x.generate(gc, ty)),
            LLVMGenerator::IsUniqueFunctionBody(x) => Some(x.generate(gc, ty)),
            LLVMGenerator::IntNegBody(x) => Some(x.generate(gc, ty)),
            LLVMGenerator::FloatNegBody(x) => Some(x.generate(gc, ty)),
            LLVMGenerator::BoolNegBody(x) => Some(x.generate(gc, ty)),
            LLVMGenerator::IntEqBody(x) => Some(x.generate(gc, ty)),
            LLVMGenerator::PtrEqBody(x) => Some(x.generate(gc, ty)),
            LLVMGenerator::FloatEqBody(x) => Some(x.generate(gc, ty)),
            LLVMGenerator::IntLessThanBody(x) => Some(x.generate(gc, ty)),
            LLVMGenerator::FloatLessThanBody(x) => Some(x.generate(gc, ty)),
            LLVMGenerator::IntLessThanOrEqBody(x) => Some(x.generate(gc, ty)),
            LLVMGenerator::FloatLessThanOrEqBody(x) => Some(x.generate(gc, ty)),
            LLVMGenerator::IntAddBody(x) => Some(x.generate(gc, ty)),
            LLVMGenerator::FloatAddBody(x) => Some(x.generate(gc, ty)),
            LLVMGenerator::IntSubBody(x) => Some(x.generate(gc, ty)),
            LLVMGenerator::FloatSubBody(x) => Some(x.generate(gc, ty)),
            LLVMGenerator::IntMulBody(x) => Some(x.generate(gc, ty)),
            LLVMGenerator::FloatMulBody(x) => Some(x.generate(gc, ty)),
            LLVMGenerator::IntDivBody(x) => Some(x.generate(gc, ty)),
            LLVMGenerator::FloatDivBody(x) => Some(x.generate(gc, ty)),
            LLVMGenerator::IntRemBody(x) => Some(x.generate(gc, ty)),
            LLVMGenerator::GetRetainedPtrOfBoxedValueFunctionBody(x) => Some(x.generate(gc, ty)),
            LLVMGenerator::MarkThreadedFunctionBody(x) => Some(x.generate(gc, ty)),
            LLVMGenerator::GetReleaseFunctionOfBoxedValueFunctionBody(x) => {
                Some(x.generate(gc, ty))
            }
            LLVMGenerator::GetBoxedValueFromRetainedPtrFunctionBody(x) => Some(x.generate(gc, ty)),
            LLVMGenerator::GetRetainFunctionOfBoxedValueFunctionBody(x) => Some(x.generate(gc, ty)),
            LLVMGenerator::GetBoxedDataPtrFunctionBody(x) => Some(x.generate(gc, ty)),
            LLVMGenerator::StructPunchBody(x) => Some(x.generate(gc, ty)),
            LLVMGenerator::StructPlugInBody(x) => Some(x.generate(gc, ty)),
            LLVMGenerator::WithRetainedFunctionBody(x) => Some(x.generate(gc, ty)),
            LLVMGenerator::UnsafeMutateBoxedInternalBody(x) => Some(x.generate(gc, ty)),
            LLVMGenerator::UnsafeMutateBoxedIOSInternalBody(x) => Some(x.generate(gc, ty)),
            LLVMGenerator::ArrayUnsafeGetLinearBoundsUncheckedUnretained(x) => {
                Some(x.generate(gc, ty))
            }
            LLVMGenerator::IOStateUnsafeCreate(x) => Some(x.generate(gc, ty)),
        };
        match obj {
            None => {
                // If the object is None, the it is already returned since `tail` is true.
                assert!(tail);
                None
            }
            Some(obj) => {
                // If the object has not been returned yet,
                if tail {
                    // If tail, then build the return instruction.
                    gc.build_tail(obj, true);
                    None
                } else {
                    Some(obj)
                }
            }
        }
    }

    pub fn free_vars(&self) -> Vec<FullName> {
        self.clone()
            .free_vars_mut()
            .into_iter()
            .map(|name| (*name).clone())
            .collect()
    }

    pub fn free_vars_mut(&mut self) -> Vec<&mut FullName> {
        match self {
            LLVMGenerator::IntLit(x) => x.free_vars(),
            LLVMGenerator::FloatLit(x) => x.free_vars(),
            LLVMGenerator::NullPtrLit(x) => x.free_vars(),
            LLVMGenerator::BoolLit(x) => x.free_vars(),
            LLVMGenerator::StringBuf(x) => x.free_vars(),
            LLVMGenerator::FixBody(x) => x.free_vars(),
            LLVMGenerator::CastIntegralBody(x) => x.free_vars(),
            LLVMGenerator::CastFloatBody(x) => x.free_vars(),
            LLVMGenerator::CastIntToFloatBody(x) => x.free_vars(),
            LLVMGenerator::CastFloatToIntBody(x) => x.free_vars(),
            LLVMGenerator::ShiftBody(x) => x.free_vars(),
            LLVMGenerator::BitwiseOperationBody(x) => x.free_vars(),
            LLVMGenerator::BitNotBody(x) => x.free_vars(),
            LLVMGenerator::FillArrayBody(x) => x.free_vars(),
            LLVMGenerator::MakeEmptyArrayBody(x) => x.free_vars(),
            LLVMGenerator::ArrayUnsafeSetBoundsUniquenessUncheckedUnreleased(x) => x.free_vars(),
            LLVMGenerator::ArrayUnsafeGetBoundsUnchecked(x) => x.free_vars(),
            LLVMGenerator::ArrayUnsafeSetSizeBody(x) => x.free_vars(),
            LLVMGenerator::ArrayGetBody(x) => x.free_vars(),
            LLVMGenerator::ArraySetBody(x) => x.free_vars(),
            LLVMGenerator::ArrayForceUniqueBody(x) => x.free_vars(),
            LLVMGenerator::ArrayGetPtrBody(x) => x.free_vars(),
            LLVMGenerator::ArrayGetSizeBody(x) => x.free_vars(),
            LLVMGenerator::ArrayGetCapacityBody(x) => x.free_vars(),
            LLVMGenerator::StructGetBody(x) => x.free_vars(),
            LLVMGenerator::StructSetBody(x) => x.free_vars(),
            LLVMGenerator::StructPunchBody(x) => x.free_vars(),
            LLVMGenerator::StructPlugInBody(x) => x.free_vars(),
            LLVMGenerator::MakeUnionBody(x) => x.free_vars(),
            LLVMGenerator::UnionAsBody(x) => x.free_vars(),
            LLVMGenerator::UnionIsBody(x) => x.free_vars(),
            LLVMGenerator::UnionModBody(x) => x.free_vars(),
            LLVMGenerator::UndefinedFunctionBody(x) => x.free_vars(),
            LLVMGenerator::IsUniqueFunctionBody(x) => x.free_vars(),
            LLVMGenerator::IntNegBody(x) => x.free_vars(),
            LLVMGenerator::FloatNegBody(x) => x.free_vars(),
            LLVMGenerator::BoolNegBody(x) => x.free_vars(),
            LLVMGenerator::IntEqBody(x) => x.free_vars(),
            LLVMGenerator::PtrEqBody(x) => x.free_vars(),
            LLVMGenerator::FloatEqBody(x) => x.free_vars(),
            LLVMGenerator::IntLessThanBody(x) => x.free_vars(),
            LLVMGenerator::FloatLessThanBody(x) => x.free_vars(),
            LLVMGenerator::IntLessThanOrEqBody(x) => x.free_vars(),
            LLVMGenerator::FloatLessThanOrEqBody(x) => x.free_vars(),
            LLVMGenerator::IntAddBody(x) => x.free_vars(),
            LLVMGenerator::FloatAddBody(x) => x.free_vars(),
            LLVMGenerator::IntSubBody(x) => x.free_vars(),
            LLVMGenerator::FloatSubBody(x) => x.free_vars(),
            LLVMGenerator::IntMulBody(x) => x.free_vars(),
            LLVMGenerator::FloatMulBody(x) => x.free_vars(),
            LLVMGenerator::IntDivBody(x) => x.free_vars(),
            LLVMGenerator::FloatDivBody(x) => x.free_vars(),
            LLVMGenerator::IntRemBody(x) => x.free_vars(),
            LLVMGenerator::MarkThreadedFunctionBody(x) => x.free_vars(),
            LLVMGenerator::GetRetainedPtrOfBoxedValueFunctionBody(x) => x.free_vars(),
            LLVMGenerator::GetBoxedValueFromRetainedPtrFunctionBody(x) => x.free_vars(),
            LLVMGenerator::GetReleaseFunctionOfBoxedValueFunctionBody(x) => x.free_vars(),
            LLVMGenerator::GetRetainFunctionOfBoxedValueFunctionBody(x) => x.free_vars(),
            LLVMGenerator::GetBoxedDataPtrFunctionBody(x) => x.free_vars(),
            LLVMGenerator::WithRetainedFunctionBody(x) => x.free_vars(),
            LLVMGenerator::UnsafeMutateBoxedInternalBody(x) => x.free_vars(),
            LLVMGenerator::UnsafeMutateBoxedIOSInternalBody(x) => x.free_vars(),
            LLVMGenerator::ArrayUnsafeGetLinearBoundsUncheckedUnretained(x) => x.free_vars(),
            LLVMGenerator::IOStateUnsafeCreate(x) => x.free_vars(),
        }
    }

    pub fn name(&self) -> String {
        let raw_name = match self {
            LLVMGenerator::IsUniqueFunctionBody(x) => x.name(),
            LLVMGenerator::IntNegBody(x) => x.name(),
            LLVMGenerator::FloatNegBody(x) => x.name(),
            LLVMGenerator::BoolNegBody(x) => x.name(),
            LLVMGenerator::IntEqBody(x) => x.name(),
            LLVMGenerator::PtrEqBody(x) => x.name(),
            LLVMGenerator::FloatEqBody(x) => x.name(),
            LLVMGenerator::IntLessThanBody(x) => x.name(),
            LLVMGenerator::FloatLessThanBody(x) => x.name(),
            LLVMGenerator::IntLessThanOrEqBody(x) => x.name(),
            LLVMGenerator::FloatLessThanOrEqBody(x) => x.name(),
            LLVMGenerator::IntAddBody(x) => x.name(),
            LLVMGenerator::FloatAddBody(x) => x.name(),
            LLVMGenerator::IntSubBody(x) => x.name(),
            LLVMGenerator::FloatSubBody(x) => x.name(),
            LLVMGenerator::IntMulBody(x) => x.name(),
            LLVMGenerator::FloatMulBody(x) => x.name(),
            LLVMGenerator::IntDivBody(x) => x.name(),
            LLVMGenerator::FloatDivBody(x) => x.name(),
            LLVMGenerator::IntRemBody(x) => x.name(),
            LLVMGenerator::MarkThreadedFunctionBody(x) => x.name(),
            LLVMGenerator::GetRetainedPtrOfBoxedValueFunctionBody(x) => x.name(),
            LLVMGenerator::GetBoxedValueFromRetainedPtrFunctionBody(x) => x.name(),
            LLVMGenerator::GetReleaseFunctionOfBoxedValueFunctionBody(x) => x.name(),
            LLVMGenerator::GetRetainFunctionOfBoxedValueFunctionBody(x) => x.name(),
            LLVMGenerator::GetBoxedDataPtrFunctionBody(x) => x.name(),
            LLVMGenerator::WithRetainedFunctionBody(x) => x.name(),
            LLVMGenerator::UnsafeMutateBoxedInternalBody(x) => x.name(),
            LLVMGenerator::UnsafeMutateBoxedIOSInternalBody(x) => x.name(),
            LLVMGenerator::ArrayUnsafeGetLinearBoundsUncheckedUnretained(x) => x.name(),
            LLVMGenerator::IOStateUnsafeCreate(x) => x.name(),
            LLVMGenerator::IntLit(x) => x.name(),
            LLVMGenerator::FloatLit(x) => x.name(),
            LLVMGenerator::NullPtrLit(x) => x.name(),
            LLVMGenerator::BoolLit(x) => x.name(),
            LLVMGenerator::StringBuf(x) => x.name(),
            LLVMGenerator::FixBody(x) => x.name(),
            LLVMGenerator::CastIntegralBody(x) => x.name(),
            LLVMGenerator::CastFloatBody(x) => x.name(),
            LLVMGenerator::CastIntToFloatBody(x) => x.name(),
            LLVMGenerator::CastFloatToIntBody(x) => x.name(),
            LLVMGenerator::ShiftBody(x) => x.name(),
            LLVMGenerator::BitwiseOperationBody(x) => x.name(),
            LLVMGenerator::BitNotBody(x) => x.name(),
            LLVMGenerator::FillArrayBody(x) => x.name(),
            LLVMGenerator::MakeEmptyArrayBody(x) => x.name(),
            LLVMGenerator::ArrayUnsafeSetBoundsUniquenessUncheckedUnreleased(x) => x.name(),
            LLVMGenerator::ArrayUnsafeGetBoundsUnchecked(x) => x.name(),
            LLVMGenerator::ArrayUnsafeSetSizeBody(x) => x.name(),
            LLVMGenerator::ArrayGetBody(x) => x.name(),
            LLVMGenerator::ArraySetBody(x) => x.name(),
            LLVMGenerator::ArrayForceUniqueBody(x) => x.name(),
            LLVMGenerator::ArrayGetPtrBody(x) => x.name(),
            LLVMGenerator::ArrayGetSizeBody(x) => x.name(),
            LLVMGenerator::ArrayGetCapacityBody(x) => x.name(),
            LLVMGenerator::StructGetBody(x) => x.name(),
            LLVMGenerator::StructSetBody(x) => x.name(),
            LLVMGenerator::StructPunchBody(x) => x.name(),
            LLVMGenerator::StructPlugInBody(x) => x.name(),
            LLVMGenerator::MakeUnionBody(x) => x.name(),
            LLVMGenerator::UnionAsBody(x) => x.name(),
            LLVMGenerator::UnionIsBody(x) => x.name(),
            LLVMGenerator::UnionModBody(x) => x.name(),
            LLVMGenerator::UndefinedFunctionBody(x) => x.name(),
        };
        format!("LLVM<{}>", raw_name)
    }

    pub fn is_primitve_literal(&self) -> bool {
        match self {
            LLVMGenerator::IntLit(_) => true,
            LLVMGenerator::FloatLit(_) => true,
            LLVMGenerator::NullPtrLit(_) => true,
            LLVMGenerator::BoolLit(_) => true,
            LLVMGenerator::StringBuf(_) => true,
            _ => false,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVM {
    pub generator: LLVMGenerator,
    // The type of this LLVM expression.
    //
    // For example, in `@ : I64 -> Array a -> a = |i, arr| LLVM<Array::@(i, arr)>;`, the `generic_ty` of the InlineLLVM `LLVM<arr.Array::@(i, arr)>` is `a`.
    // Note that `generic_ty` may contain type variables, and it is not changed in type instantiation.
    pub generic_ty: Arc<TypeNode>,
}
