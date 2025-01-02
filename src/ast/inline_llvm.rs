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
    StringLit(InlineLLVMStringLit),
    FixBody(InlineLLVMFixBody),
    CastIntegralBody(InlineLLVMCastIntegralBody),
    CastFloatBody(InlineLLVMCastFloatBody),
    CastIntToFloatBody(InlineLLVMCastIntToFloatBody),
    CastFloatToIntBody(InlineLLVMCastFloatToIntBody),
    ShiftBody(InlineLLVMShiftBody),
    BitwiseOperationBody(InlineLLVMBitwiseOperationBody),
    FillArrayBody(InlineLLVMFillArrayBody),
    MakeEmptyArrayBody(InlineLLVMMakeEmptyArrayBody),
    ArrayUnsafeSetBody(InlineLLVMArrayUnsafeSetBody),
    ArrayUnsafeGetBody(InlineLLVMArrayUnsafeGetBody),
    ArrayUnsafeSetSizeBody(InlineLLVMArrayUnsafeSetSizeBody),
    ArrayGetBody(InlineLLVMArrayGetBody),
    ArraySetBody(InlineLLVMArraySetBody),
    ArrayModBody(InlineLLVMArrayModBody),
    ArrayForceUniqueBody(InlineLLVMArrayForceUniqueBody),
    ArrayGetPtrBody(InlineLLVMArrayGetPtrBody),
    ArrayGetSizeBody(InlineLLVMArrayGetSizeBody),
    ArrayGetCapacityBody(InlineLLVMArrayGetCapacityBody),
    StructGetBody(InlineLLVMStructGetBody),
    StructModBody(InlineLLVMStructModBody),
    StructSetBody(InlineLLVMStructSetBody),
    StructPunchBody(InlineLLVMStructPunchBody),
    StructPlugInBody(InlineLLVMStructPlugInBody),
    MakeUnionBody(InlineLLVMMakeUnionBody),
    UnionAsBody(InlineLLVMUnionAsBody),
    UnionIsBody(InlineLLVMUnionIsBody),
    UnionModBody(InlineLLVMUnionModBody),
    UndefinedFunctionBody(InlineLLVMUndefinedFunctionBody),
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
    UnsafeMutateBoxedDataFunctionBody(InlineLLVMUnsafeMutateBoxedDataFunctionBody),
    UnsafeMutateBoxedDataIOStateFunctionBody(InlineLLVMUnsafeMutateBoxedDataIOStateFunctionBody),
    ArrayUnsafeGetLinearFunctionBody(InlineLLVMArrayUnsafeGetLinearFunctionBody),
    UnsafePerformFunctionBody(InlineLLVMUnsafePerformFunctionBody),
}

impl LLVMGenerator {
    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        ty: &Arc<TypeNode>,
        bvs: &Vec<FullName>, // borrowed variables
        tail: bool,
    ) -> Option<Object<'c>> {
        let obj = match self {
            LLVMGenerator::IntLit(x) => Some(x.generate(gc, ty, bvs)),
            LLVMGenerator::FloatLit(x) => Some(x.generate(gc, ty, bvs)),
            LLVMGenerator::NullPtrLit(x) => Some(x.generate(gc, ty, bvs)),
            LLVMGenerator::BoolLit(x) => Some(x.generate(gc, ty, bvs)),
            LLVMGenerator::StringLit(x) => Some(x.generate(gc, ty, bvs)),
            LLVMGenerator::FixBody(x) => x.generate(gc, ty, bvs, tail),
            LLVMGenerator::CastIntegralBody(x) => Some(x.generate(gc, ty, bvs)),
            LLVMGenerator::CastFloatBody(x) => Some(x.generate(gc, ty, bvs)),
            LLVMGenerator::CastIntToFloatBody(x) => Some(x.generate(gc, ty, bvs)),
            LLVMGenerator::CastFloatToIntBody(x) => Some(x.generate(gc, ty, bvs)),
            LLVMGenerator::ShiftBody(x) => Some(x.generate(gc, ty, bvs)),
            LLVMGenerator::BitwiseOperationBody(x) => Some(x.generate(gc, ty, bvs)),
            LLVMGenerator::FillArrayBody(x) => Some(x.generate(gc, ty, bvs)),
            LLVMGenerator::MakeEmptyArrayBody(x) => Some(x.generate(gc, ty, bvs)),
            LLVMGenerator::ArrayUnsafeSetBody(x) => Some(x.generate(gc, ty, bvs)),
            LLVMGenerator::ArrayUnsafeGetBody(x) => Some(x.generate(gc, ty, bvs)),
            LLVMGenerator::ArrayUnsafeSetSizeBody(x) => Some(x.generate(gc, ty, bvs)),
            LLVMGenerator::ArrayGetBody(x) => Some(x.generate(gc, ty, bvs)),
            LLVMGenerator::ArraySetBody(x) => Some(x.generate(gc, ty, bvs)),
            LLVMGenerator::ArrayModBody(x) => Some(x.generate(gc, ty, bvs)),
            LLVMGenerator::ArrayForceUniqueBody(x) => Some(x.generate(gc, ty, bvs)),
            LLVMGenerator::ArrayGetPtrBody(x) => Some(x.generate(gc, ty, bvs)),
            LLVMGenerator::ArrayGetSizeBody(x) => Some(x.generate(gc, ty, bvs)),
            LLVMGenerator::ArrayGetCapacityBody(x) => Some(x.generate(gc, ty, bvs)),
            LLVMGenerator::StructGetBody(x) => Some(x.generate(gc, ty, bvs)),
            LLVMGenerator::StructModBody(x) => Some(x.generate(gc, ty, bvs)),
            LLVMGenerator::StructSetBody(x) => Some(x.generate(gc, ty, bvs)),
            LLVMGenerator::MakeUnionBody(x) => Some(x.generate(gc, ty, bvs)),
            LLVMGenerator::UnionAsBody(x) => Some(x.generate(gc, ty, bvs)),
            LLVMGenerator::UnionIsBody(x) => Some(x.generate(gc, ty, bvs)),
            LLVMGenerator::UnionModBody(x) => Some(x.generate(gc, ty, bvs)),
            LLVMGenerator::UndefinedFunctionBody(x) => Some(x.generate(gc, ty, bvs)),
            LLVMGenerator::IsUniqueFunctionBody(x) => Some(x.generate(gc, ty, bvs)),
            LLVMGenerator::IntNegBody(x) => Some(x.generate(gc, ty, bvs)),
            LLVMGenerator::FloatNegBody(x) => Some(x.generate(gc, ty, bvs)),
            LLVMGenerator::BoolNegBody(x) => Some(x.generate(gc, ty, bvs)),
            LLVMGenerator::IntEqBody(x) => Some(x.generate(gc, ty, bvs)),
            LLVMGenerator::PtrEqBody(x) => Some(x.generate(gc, ty, bvs)),
            LLVMGenerator::FloatEqBody(x) => Some(x.generate(gc, ty, bvs)),
            LLVMGenerator::IntLessThanBody(x) => Some(x.generate(gc, ty, bvs)),
            LLVMGenerator::FloatLessThanBody(x) => Some(x.generate(gc, ty, bvs)),
            LLVMGenerator::IntLessThanOrEqBody(x) => Some(x.generate(gc, ty, bvs)),
            LLVMGenerator::FloatLessThanOrEqBody(x) => Some(x.generate(gc, ty, bvs)),
            LLVMGenerator::IntAddBody(x) => Some(x.generate(gc, ty, bvs)),
            LLVMGenerator::FloatAddBody(x) => Some(x.generate(gc, ty, bvs)),
            LLVMGenerator::IntSubBody(x) => Some(x.generate(gc, ty, bvs)),
            LLVMGenerator::FloatSubBody(x) => Some(x.generate(gc, ty, bvs)),
            LLVMGenerator::IntMulBody(x) => Some(x.generate(gc, ty, bvs)),
            LLVMGenerator::FloatMulBody(x) => Some(x.generate(gc, ty, bvs)),
            LLVMGenerator::IntDivBody(x) => Some(x.generate(gc, ty, bvs)),
            LLVMGenerator::FloatDivBody(x) => Some(x.generate(gc, ty, bvs)),
            LLVMGenerator::IntRemBody(x) => Some(x.generate(gc, ty, bvs)),
            LLVMGenerator::GetRetainedPtrOfBoxedValueFunctionBody(x) => {
                Some(x.generate(gc, ty, bvs))
            }
            LLVMGenerator::MarkThreadedFunctionBody(x) => Some(x.generate(gc, ty, bvs)),
            LLVMGenerator::GetReleaseFunctionOfBoxedValueFunctionBody(x) => {
                Some(x.generate(gc, ty, bvs))
            }
            LLVMGenerator::GetBoxedValueFromRetainedPtrFunctionBody(x) => {
                Some(x.generate(gc, ty, bvs))
            }
            LLVMGenerator::GetRetainFunctionOfBoxedValueFunctionBody(x) => {
                Some(x.generate(gc, ty, bvs))
            }
            LLVMGenerator::GetBoxedDataPtrFunctionBody(x) => Some(x.generate(gc, ty, bvs)),
            LLVMGenerator::StructPunchBody(x) => Some(x.generate(gc, ty, bvs)),
            LLVMGenerator::StructPlugInBody(x) => Some(x.generate(gc, ty, bvs)),
            LLVMGenerator::WithRetainedFunctionBody(x) => Some(x.generate(gc, ty, bvs)),
            LLVMGenerator::UnsafeMutateBoxedDataFunctionBody(x) => Some(x.generate(gc, ty, bvs)),
            LLVMGenerator::UnsafeMutateBoxedDataIOStateFunctionBody(x) => {
                Some(x.generate(gc, ty, bvs))
            }
            LLVMGenerator::ArrayUnsafeGetLinearFunctionBody(x) => Some(x.generate(gc, ty, bvs)),
            LLVMGenerator::UnsafePerformFunctionBody(x) => Some(x.generate(gc, ty, bvs)),
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

    // Returns a list of variables which is released by this generator.
    // None if the generator does not support this interface.
    // This is used for the borrowing optimization.
    pub fn released_vars(&self) -> Option<Vec<FullName>> {
        match self {
            LLVMGenerator::ArrayGetBody(x) => Some(x.released_vars()),
            LLVMGenerator::ArrayGetSizeBody(x) => Some(x.released_vars()),
            LLVMGenerator::ArrayUnsafeGetBody(x) => Some(x.released_vars()),
            LLVMGenerator::ArrayGetPtrBody(x) => Some(x.released_vars()),
            LLVMGenerator::ArrayGetCapacityBody(x) => Some(x.released_vars()),
            _ => None,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVM {
    pub generator: LLVMGenerator,
    pub free_vars: Vec<FullName>, // e.g. "+" literal has two free variables.
    // Set of variables which is contained in the list `released_vars()` but should not be released.
    pub borrowed_vars: Vec<FullName>,
    pub name: String,
    pub ty: Arc<TypeNode>,
}
