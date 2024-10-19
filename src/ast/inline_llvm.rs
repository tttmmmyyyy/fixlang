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
    LoopFunctionBody(InlineLLVMLoopFunctionBody),
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
    DoWithRetainedFunctionBody(InlineLLVMDoWithRetainedFunctionBody),
    UnsafeMutateBoxedDataFunctionBody(InlineLLVMUnsafeMutateBoxedDataFunctionBody),
}

impl LLVMGenerator {
    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        ty: &Arc<TypeNode>,
        rvo: Option<Object<'c>>,
        bvs: &Vec<FullName>, // borrowed variables
    ) -> Object<'c> {
        match self {
            LLVMGenerator::IntLit(x) => x.generate(gc, ty, rvo, bvs),
            LLVMGenerator::FloatLit(x) => x.generate(gc, ty, rvo, bvs),
            LLVMGenerator::NullPtrLit(x) => x.generate(gc, ty, rvo, bvs),
            LLVMGenerator::BoolLit(x) => x.generate(gc, ty, rvo, bvs),
            LLVMGenerator::StringLit(x) => x.generate(gc, ty, rvo, bvs),
            LLVMGenerator::FixBody(x) => x.generate(gc, ty, rvo, bvs),
            LLVMGenerator::CastIntegralBody(x) => x.generate(gc, ty, rvo, bvs),
            LLVMGenerator::CastFloatBody(x) => x.generate(gc, ty, rvo, bvs),
            LLVMGenerator::CastIntToFloatBody(x) => x.generate(gc, ty, rvo, bvs),
            LLVMGenerator::CastFloatToIntBody(x) => x.generate(gc, ty, rvo, bvs),
            LLVMGenerator::ShiftBody(x) => x.generate(gc, ty, rvo, bvs),
            LLVMGenerator::BitwiseOperationBody(x) => x.generate(gc, ty, rvo, bvs),
            LLVMGenerator::FillArrayBody(x) => x.generate(gc, ty, rvo, bvs),
            LLVMGenerator::MakeEmptyArrayBody(x) => x.generate(gc, ty, rvo, bvs),
            LLVMGenerator::ArrayUnsafeSetBody(x) => x.generate(gc, ty, rvo, bvs),
            LLVMGenerator::ArrayUnsafeGetBody(x) => x.generate(gc, ty, rvo, bvs),
            LLVMGenerator::ArrayUnsafeSetSizeBody(x) => x.generate(gc, ty, rvo, bvs),
            LLVMGenerator::ArrayGetBody(x) => x.generate(gc, ty, rvo, bvs),
            LLVMGenerator::ArraySetBody(x) => x.generate(gc, ty, rvo, bvs),
            LLVMGenerator::ArrayModBody(x) => x.generate(gc, ty, rvo, bvs),
            LLVMGenerator::ArrayForceUniqueBody(x) => x.generate(gc, ty, rvo, bvs),
            LLVMGenerator::ArrayGetPtrBody(x) => x.generate(gc, ty, rvo, bvs),
            LLVMGenerator::ArrayGetSizeBody(x) => x.generate(gc, ty, rvo, bvs),
            LLVMGenerator::ArrayGetCapacityBody(x) => x.generate(gc, ty, rvo, bvs),
            LLVMGenerator::StructGetBody(x) => x.generate(gc, ty, rvo, bvs),
            LLVMGenerator::StructModBody(x) => x.generate(gc, ty, rvo, bvs),
            LLVMGenerator::StructSetBody(x) => x.generate(gc, ty, rvo, bvs),
            LLVMGenerator::MakeUnionBody(x) => x.generate(gc, ty, rvo, bvs),
            LLVMGenerator::UnionAsBody(x) => x.generate(gc, ty, rvo, bvs),
            LLVMGenerator::UnionIsBody(x) => x.generate(gc, ty, rvo, bvs),
            LLVMGenerator::UnionModBody(x) => x.generate(gc, ty, rvo, bvs),
            LLVMGenerator::LoopFunctionBody(x) => x.generate(gc, ty, rvo, bvs),
            LLVMGenerator::UndefinedFunctionBody(x) => x.generate(gc, ty, rvo, bvs),
            LLVMGenerator::IsUniqueFunctionBody(x) => x.generate(gc, ty, rvo, bvs),
            LLVMGenerator::IntNegBody(x) => x.generate(gc, ty, rvo, bvs),
            LLVMGenerator::FloatNegBody(x) => x.generate(gc, ty, rvo, bvs),
            LLVMGenerator::BoolNegBody(x) => x.generate(gc, ty, rvo, bvs),
            LLVMGenerator::IntEqBody(x) => x.generate(gc, ty, rvo, bvs),
            LLVMGenerator::PtrEqBody(x) => x.generate(gc, ty, rvo, bvs),
            LLVMGenerator::FloatEqBody(x) => x.generate(gc, ty, rvo, bvs),
            LLVMGenerator::IntLessThanBody(x) => x.generate(gc, ty, rvo, bvs),
            LLVMGenerator::FloatLessThanBody(x) => x.generate(gc, ty, rvo, bvs),
            LLVMGenerator::IntLessThanOrEqBody(x) => x.generate(gc, ty, rvo, bvs),
            LLVMGenerator::FloatLessThanOrEqBody(x) => x.generate(gc, ty, rvo, bvs),
            LLVMGenerator::IntAddBody(x) => x.generate(gc, ty, rvo, bvs),
            LLVMGenerator::FloatAddBody(x) => x.generate(gc, ty, rvo, bvs),
            LLVMGenerator::IntSubBody(x) => x.generate(gc, ty, rvo, bvs),
            LLVMGenerator::FloatSubBody(x) => x.generate(gc, ty, rvo, bvs),
            LLVMGenerator::IntMulBody(x) => x.generate(gc, ty, rvo, bvs),
            LLVMGenerator::FloatMulBody(x) => x.generate(gc, ty, rvo, bvs),
            LLVMGenerator::IntDivBody(x) => x.generate(gc, ty, rvo, bvs),
            LLVMGenerator::FloatDivBody(x) => x.generate(gc, ty, rvo, bvs),
            LLVMGenerator::IntRemBody(x) => x.generate(gc, ty, rvo, bvs),
            LLVMGenerator::GetRetainedPtrOfBoxedValueFunctionBody(x) => {
                x.generate(gc, ty, rvo, bvs)
            }
            LLVMGenerator::MarkThreadedFunctionBody(x) => x.generate(gc, ty, rvo, bvs),
            LLVMGenerator::GetReleaseFunctionOfBoxedValueFunctionBody(x) => {
                x.generate(gc, ty, rvo, bvs)
            }
            LLVMGenerator::GetBoxedValueFromRetainedPtrFunctionBody(x) => {
                x.generate(gc, ty, rvo, bvs)
            }
            LLVMGenerator::GetRetainFunctionOfBoxedValueFunctionBody(x) => {
                x.generate(gc, ty, rvo, bvs)
            }
            LLVMGenerator::GetBoxedDataPtrFunctionBody(x) => x.generate(gc, ty, rvo, bvs),
            LLVMGenerator::StructPunchBody(x) => x.generate(gc, ty, rvo, bvs),
            LLVMGenerator::StructPlugInBody(x) => x.generate(gc, ty, rvo, bvs),
            LLVMGenerator::DoWithRetainedFunctionBody(x) => x.generate(gc, ty, rvo, bvs),
            LLVMGenerator::UnsafeMutateBoxedDataFunctionBody(x) => x.generate(gc, ty, rvo, bvs),
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
