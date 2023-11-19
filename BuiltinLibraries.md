# Table of contents

- [Table of contents](#table-of-contents)
- [NOTE](#note)
- [Module `Std`](#module-std)
  - [Types, related values and implementations](#types-related-values-and-implementations)
    - [Bool](#bool)
      - [`impl Bool : Eq`](#impl-bool--eq)
      - [`impl Bool : ToString`](#impl-bool--tostring)
    - [I8](#i8)
      - [`abs : I8 -> I8`](#abs--i8---i8)
      - [`maximum : I8`](#maximum--i8)
      - [`minimum : I8`](#minimum--i8)
      - [`bit_and : I8 -> I8 -> I8`](#bit_and--i8---i8---i8)
      - [`bit_or : I8 -> I8 -> I8`](#bit_or--i8---i8---i8)
      - [`bit_xor : I8 -> I8 -> I8`](#bit_xor--i8---i8---i8)
      - [`shift_left : I8 -> I8 -> I8`](#shift_left--i8---i8---i8)
      - [`shift_right : I8 -> I8 -> I8`](#shift_right--i8---i8---i8)
      - [`to_I8 : I8 -> I8`](#to_i8--i8---i8)
      - [`to_U8 : I8 -> U8`](#to_u8--i8---u8)
      - [`to_I16 : I8 -> I16`](#to_i16--i8---i16)
      - [`to_U16 : I8 -> U16`](#to_u16--i8---u16)
      - [`to_I32 : I8 -> I32`](#to_i32--i8---i32)
      - [`to_U32 : I8 -> U32`](#to_u32--i8---u32)
      - [`to_I64 : I8 -> I64`](#to_i64--i8---i64)
      - [`to_U64 : I8 -> U64`](#to_u64--i8---u64)
      - [`to_F32 : I8 -> F32`](#to_f32--i8---f32)
      - [`to_F64 : I8 -> F64`](#to_f64--i8---f64)
      - [`impl I8 : Add`](#impl-i8--add)
      - [`impl I8 : Eq`](#impl-i8--eq)
      - [`impl I8 : FromBytes`](#impl-i8--frombytes)
      - [`impl I8 : FromString`](#impl-i8--fromstring)
      - [`impl I8 : LessThan`](#impl-i8--lessthan)
      - [`impl I8 : LessThanOrEq`](#impl-i8--lessthanoreq)
      - [`impl I8 : Mul`](#impl-i8--mul)
      - [`impl I8 : Neg`](#impl-i8--neg)
      - [`impl I8 : Rem`](#impl-i8--rem)
      - [`impl I8 : Sub`](#impl-i8--sub)
      - [`impl I8 : ToBytes`](#impl-i8--tobytes)
      - [`impl I8 : ToString`](#impl-i8--tostring)
      - [`impl I8 : Zero`](#impl-i8--zero)
    - [U8](#u8)
      - [`maximum : U8`](#maximum--u8)
      - [`minimum : U8`](#minimum--u8)
      - [`bit_and : U8 -> U8 -> U8`](#bit_and--u8---u8---u8)
      - [`bit_or : U8 -> U8 -> U8`](#bit_or--u8---u8---u8)
      - [`bit_xor : U8 -> U8 -> U8`](#bit_xor--u8---u8---u8)
      - [`shift_left : U8 -> U8 -> U8`](#shift_left--u8---u8---u8)
      - [`shift_right : U8 -> U8 -> U8`](#shift_right--u8---u8---u8)
      - [`to_I8 : U8 -> I8`](#to_i8--u8---i8)
      - [`to_U8 : U8 -> U8`](#to_u8--u8---u8)
      - [`to_I16 : U8 -> I16`](#to_i16--u8---i16)
      - [`to_U16 : U8 -> U16`](#to_u16--u8---u16)
      - [`to_I32 : U8 -> I32`](#to_i32--u8---i32)
      - [`to_U32 : U8 -> U32`](#to_u32--u8---u32)
      - [`to_I64 : U8 -> I64`](#to_i64--u8---i64)
      - [`to_U64 : U8 -> U64`](#to_u64--u8---u64)
      - [`to_F32 : U8 -> F32`](#to_f32--u8---f32)
      - [`to_F64 : U8 -> F64`](#to_f64--u8---f64)
      - [`impl U8 : Add`](#impl-u8--add)
      - [`impl U8 : Eq`](#impl-u8--eq)
      - [`impl U8 : FromBytes`](#impl-u8--frombytes)
      - [`impl U8 : FromString`](#impl-u8--fromstring)
      - [`impl U8 : LessThan`](#impl-u8--lessthan)
      - [`impl U8 : LessThanOrEq`](#impl-u8--lessthanoreq)
      - [`impl U8 : Mul`](#impl-u8--mul)
      - [`impl U8 : Neg`](#impl-u8--neg)
      - [`impl U8 : Rem`](#impl-u8--rem)
      - [`impl U8 : Sub`](#impl-u8--sub)
      - [`impl U8 : ToBytes`](#impl-u8--tobytes)
      - [`impl U8 : ToString`](#impl-u8--tostring)
      - [`impl U8 : Zero`](#impl-u8--zero)
    - [I16](#i16)
      - [`abs : I16 -> I16`](#abs--i16---i16)
      - [`maximum : I16`](#maximum--i16)
      - [`minimum : I16`](#minimum--i16)
      - [`bit_and : I16 -> I16 -> I16`](#bit_and--i16---i16---i16)
      - [`bit_or : I16 -> I16 -> I16`](#bit_or--i16---i16---i16)
      - [`bit_xor : I16 -> I16 -> I16`](#bit_xor--i16---i16---i16)
      - [`shift_left : I16 -> I16 -> I16`](#shift_left--i16---i16---i16)
      - [`shift_right : I16 -> I16 -> I16`](#shift_right--i16---i16---i16)
      - [`to_I8 : I16 -> I8`](#to_i8--i16---i8)
      - [`to_U8 : I16 -> U8`](#to_u8--i16---u8)
      - [`to_I16 : I16 -> I16`](#to_i16--i16---i16)
      - [`to_U16 : I16 -> U16`](#to_u16--i16---u16)
      - [`to_I32 : I16 -> I32`](#to_i32--i16---i32)
      - [`to_U32 : I16 -> U32`](#to_u32--i16---u32)
      - [`to_I64 : I16 -> I64`](#to_i64--i16---i64)
      - [`to_U64 : I16 -> U64`](#to_u64--i16---u64)
      - [`to_F32 : I16 -> F32`](#to_f32--i16---f32)
      - [`to_F64 : I16 -> F64`](#to_f64--i16---f64)
      - [`impl I16 : Add`](#impl-i16--add)
      - [`impl I16 : Eq`](#impl-i16--eq)
      - [`impl I16 : FromBytes`](#impl-i16--frombytes)
      - [`impl I16 : FromString`](#impl-i16--fromstring)
      - [`impl I16 : LessThan`](#impl-i16--lessthan)
      - [`impl I16 : LessThanOrEq`](#impl-i16--lessthanoreq)
      - [`impl I16 : Mul`](#impl-i16--mul)
      - [`impl I16 : Neg`](#impl-i16--neg)
      - [`impl I16 : Rem`](#impl-i16--rem)
      - [`impl I16 : Sub`](#impl-i16--sub)
      - [`impl I16 : ToBytes`](#impl-i16--tobytes)
      - [`impl I16 : ToString`](#impl-i16--tostring)
      - [`impl I16 : Zero`](#impl-i16--zero)
    - [U16](#u16)
      - [`maximum : U16`](#maximum--u16)
      - [`minimum : U16`](#minimum--u16)
      - [`bit_and : U16 -> U16 -> U16`](#bit_and--u16---u16---u16)
      - [`bit_or : U16 -> U16 -> U16`](#bit_or--u16---u16---u16)
      - [`bit_xor : U16 -> U16 -> U16`](#bit_xor--u16---u16---u16)
      - [`shift_left : U16 -> U16 -> U16`](#shift_left--u16---u16---u16)
      - [`shift_right : U16 -> U16 -> U16`](#shift_right--u16---u16---u16)
      - [`to_I8 : U16 -> I8`](#to_i8--u16---i8)
      - [`to_U8 : U16 -> U8`](#to_u8--u16---u8)
      - [`to_I16 : U16 -> I16`](#to_i16--u16---i16)
      - [`to_U16 : U16 -> U16`](#to_u16--u16---u16)
      - [`to_I32 : U16 -> I32`](#to_i32--u16---i32)
      - [`to_U32 : U16 -> U32`](#to_u32--u16---u32)
      - [`to_I64 : U16 -> I64`](#to_i64--u16---i64)
      - [`to_U64 : U16 -> U64`](#to_u64--u16---u64)
      - [`to_F32 : U16 -> F32`](#to_f32--u16---f32)
      - [`to_F64 : U16 -> F64`](#to_f64--u16---f64)
      - [`impl U16 : Add`](#impl-u16--add)
      - [`impl U16 : Eq`](#impl-u16--eq)
      - [`impl U16 : FromBytes`](#impl-u16--frombytes)
      - [`impl U16 : FromString`](#impl-u16--fromstring)
      - [`impl U16 : LessThan`](#impl-u16--lessthan)
      - [`impl U16 : LessThanOrEq`](#impl-u16--lessthanoreq)
      - [`impl U16 : Mul`](#impl-u16--mul)
      - [`impl U16 : Neg`](#impl-u16--neg)
      - [`impl U16 : Rem`](#impl-u16--rem)
      - [`impl U16 : Sub`](#impl-u16--sub)
      - [`impl U16 : ToBytes`](#impl-u16--tobytes)
      - [`impl U16 : ToString`](#impl-u16--tostring)
      - [`impl U16 : Zero`](#impl-u16--zero)
    - [I32](#i32)
      - [`abs : I32 -> I32`](#abs--i32---i32)
      - [`maximum : I32`](#maximum--i32)
      - [`minimum : I32`](#minimum--i32)
      - [`bit_and : I32 -> I32 -> I32`](#bit_and--i32---i32---i32)
      - [`bit_or : I32 -> I32 -> I32`](#bit_or--i32---i32---i32)
      - [`bit_xor : I32 -> I32 -> I32`](#bit_xor--i32---i32---i32)
      - [`shift_left : I32 -> I32 -> I32`](#shift_left--i32---i32---i32)
      - [`shift_right : I32 -> I32 -> I32`](#shift_right--i32---i32---i32)
      - [`to_I8 : I32 -> I8`](#to_i8--i32---i8)
      - [`to_U8 : I32 -> U8`](#to_u8--i32---u8)
      - [`to_I16 : I32 -> I16`](#to_i16--i32---i16)
      - [`to_U16 : I32 -> U16`](#to_u16--i32---u16)
      - [`to_U32 : I32 -> U32`](#to_u32--i32---u32)
      - [`to_I64 : I32 -> I64`](#to_i64--i32---i64)
      - [`to_U64 : I32 -> U64`](#to_u64--i32---u64)
      - [`to_F32 : I32 -> F32`](#to_f32--i32---f32)
      - [`to_F64 : I32 -> F64`](#to_f64--i32---f64)
      - [`impl I32 : Add`](#impl-i32--add)
      - [`impl I32 : Eq`](#impl-i32--eq)
      - [`impl I32 : FromBytes`](#impl-i32--frombytes)
      - [`impl I32 : FromString`](#impl-i32--fromstring)
      - [`impl I32 : LessThan`](#impl-i32--lessthan)
      - [`impl I32 : LessThanOrEq`](#impl-i32--lessthanoreq)
      - [`impl I32 : Mul`](#impl-i32--mul)
      - [`impl I32 : Neg`](#impl-i32--neg)
      - [`impl I32 : Rem`](#impl-i32--rem)
      - [`impl I32 : Sub`](#impl-i32--sub)
      - [`impl I32 : ToBytes`](#impl-i32--tobytes)
      - [`impl I32 : ToString`](#impl-i32--tostring)
      - [`impl I32 : Zero`](#impl-i32--zero)
    - [U32](#u32)
      - [`maximum : U32`](#maximum--u32)
      - [`minimum : U32`](#minimum--u32)
      - [`bit_and : U32 -> U32 -> U32`](#bit_and--u32---u32---u32)
      - [`bit_or : U32 -> U32 -> U32`](#bit_or--u32---u32---u32)
      - [`bit_xor : U32 -> U32 -> U32`](#bit_xor--u32---u32---u32)
      - [`shift_left : U32 -> U32 -> U32`](#shift_left--u32---u32---u32)
      - [`shift_right : U32 -> U32 -> U32`](#shift_right--u32---u32---u32)
      - [`to_I8 : U32 -> I8`](#to_i8--u32---i8)
      - [`to_U8 : U32 -> U8`](#to_u8--u32---u8)
      - [`to_I16 : U32 -> I16`](#to_i16--u32---i16)
      - [`to_U16 : U32 -> U16`](#to_u16--u32---u16)
      - [`to_I32 : U32 -> I32`](#to_i32--u32---i32)
      - [`to_I64 : U32 -> I64`](#to_i64--u32---i64)
      - [`to_U64 : U32 -> U64`](#to_u64--u32---u64)
      - [`to_F32 : U32 -> F32`](#to_f32--u32---f32)
      - [`to_F64 : U32 -> F64`](#to_f64--u32---f64)
      - [`impl U32 : Add`](#impl-u32--add)
      - [`impl U32 : Eq`](#impl-u32--eq)
      - [`impl U32 : FromBytes`](#impl-u32--frombytes)
      - [`impl U32 : FromString`](#impl-u32--fromstring)
      - [`impl U32 : LessThan`](#impl-u32--lessthan)
      - [`impl U32 : LessThanOrEq`](#impl-u32--lessthanoreq)
      - [`impl U32 : Mul`](#impl-u32--mul)
      - [`impl U32 : Neg`](#impl-u32--neg)
      - [`impl U32 : Rem`](#impl-u32--rem)
      - [`impl U32 : Sub`](#impl-u32--sub)
      - [`impl U32 : ToBytes`](#impl-u32--tobytes)
      - [`impl U32 : ToString`](#impl-u32--tostring)
      - [`impl U32 : Zero`](#impl-u32--zero)
    - [I64](#i64)
      - [`abs : I64 -> I64`](#abs--i64---i64)
      - [`maximum : I64`](#maximum--i64)
      - [`minimum : I64`](#minimum--i64)
      - [`bit_and : I64 -> I64 -> I64`](#bit_and--i64---i64---i64)
      - [`bit_or : I64 -> I64 -> I64`](#bit_or--i64---i64---i64)
      - [`bit_xor : I64 -> I64 -> I64`](#bit_xor--i64---i64---i64)
      - [`shift_left : I64 -> I64 -> I64`](#shift_left--i64---i64---i64)
      - [`shift_right : I64 -> I64 -> I64`](#shift_right--i64---i64---i64)
      - [`to_I8 : I64 -> I8`](#to_i8--i64---i8)
      - [`to_U8 : I64 -> U8`](#to_u8--i64---u8)
      - [`to_I16 : I64 -> I16`](#to_i16--i64---i16)
      - [`to_U16 : I64 -> U16`](#to_u16--i64---u16)
      - [`to_I32 : I64 -> I32`](#to_i32--i64---i32)
      - [`to_U32 : I64 -> U32`](#to_u32--i64---u32)
      - [`to_U64 : I64 -> U64`](#to_u64--i64---u64)
      - [`to_F32 : I64 -> F32`](#to_f32--i64---f32)
      - [`to_F64 : I64 -> F64`](#to_f64--i64---f64)
      - [`impl I64 : Add`](#impl-i64--add)
      - [`impl I64 : Eq`](#impl-i64--eq)
      - [`impl I64 : FromBytes`](#impl-i64--frombytes)
      - [`impl I64 : FromString`](#impl-i64--fromstring)
      - [`impl I64 : LessThan`](#impl-i64--lessthan)
      - [`impl I64 : LessThanOrEq`](#impl-i64--lessthanoreq)
      - [`impl I64 : Mul`](#impl-i64--mul)
      - [`impl I64 : Neg`](#impl-i64--neg)
      - [`impl I64 : Rem`](#impl-i64--rem)
      - [`impl I64 : Sub`](#impl-i64--sub)
      - [`impl I64 : ToBytes`](#impl-i64--tobytes)
      - [`impl I64 : ToString`](#impl-i64--tostring)
      - [`impl I64 : Zero`](#impl-i64--zero)
    - [U64](#u64)
      - [`maximum : U64`](#maximum--u64)
      - [`minimum : U64`](#minimum--u64)
      - [`bit_and : U64 -> U64 -> U64`](#bit_and--u64---u64---u64)
      - [`bit_or : U64 -> U64 -> U64`](#bit_or--u64---u64---u64)
      - [`bit_xor : U64 -> U64 -> U64`](#bit_xor--u64---u64---u64)
      - [`shift_left : U64 -> U64 -> U64`](#shift_left--u64---u64---u64)
      - [`shift_right : U64 -> U64 -> U64`](#shift_right--u64---u64---u64)
      - [`to_I8 : U64 -> I8`](#to_i8--u64---i8)
      - [`to_U8 : U64 -> U8`](#to_u8--u64---u8)
      - [`to_I16 : U64 -> I16`](#to_i16--u64---i16)
      - [`to_U16 : U64 -> U16`](#to_u16--u64---u16)
      - [`to_I32 : U64 -> I32`](#to_i32--u64---i32)
      - [`to_U32 : U64 -> U32`](#to_u32--u64---u32)
      - [`to_I64 : U64 -> I64`](#to_i64--u64---i64)
      - [`to_F32 : U64 -> F32`](#to_f32--u64---f32)
      - [`to_F64 : U64 -> F64`](#to_f64--u64---f64)
      - [`impl U64 : Add`](#impl-u64--add)
      - [`impl U64 : Eq`](#impl-u64--eq)
      - [`impl U64 : FromBytes`](#impl-u64--frombytes)
      - [`impl U64 : FromString`](#impl-u64--fromstring)
      - [`impl U64 : LessThan`](#impl-u64--lessthan)
      - [`impl U64 : LessThanOrEq`](#impl-u64--lessthanoreq)
      - [`impl U64 : Mul`](#impl-u64--mul)
      - [`impl U64 : Neg`](#impl-u64--neg)
      - [`impl U64 : Rem`](#impl-u64--rem)
      - [`impl U64 : Sub`](#impl-u64--sub)
      - [`impl U64 : ToBytes`](#impl-u64--tobytes)
      - [`impl U64 : ToString`](#impl-u64--tostring)
      - [`impl U64 : Zero`](#impl-u64--zero)
    - [F32](#f32)
      - [`abs : F32 -> F32`](#abs--f32---f32)
      - [`to_I8 : F32 -> I8`](#to_i8--f32---i8)
      - [`to_U8 : F32 -> U8`](#to_u8--f32---u8)
      - [`to_I16 : F32 -> I16`](#to_i16--f32---i16)
      - [`to_U16 : F32 -> U16`](#to_u16--f32---u16)
      - [`to_I32 : F32 -> I32`](#to_i32--f32---i32)
      - [`to_U32 : F32 -> U32`](#to_u32--f32---u32)
      - [`to_I64 : F32 -> I64`](#to_i64--f32---i64)
      - [`to_U64 : F32 -> U64`](#to_u64--f32---u64)
      - [`to_F32 : F32 -> F32`](#to_f32--f32---f32)
      - [`to_F64 : F32 -> F64`](#to_f64--f32---f64)
      - [`to_string_exp : F32 -> String`](#to_string_exp--f32---string)
      - [`to_string_exp_precision : U8 -> F32 -> String`](#to_string_exp_precision--u8---f32---string)
      - [`to_string_precision : U8 -> F32 -> String`](#to_string_precision--u8---f32---string)
      - [`impl F32 : Add`](#impl-f32--add)
      - [`impl F32 : Div`](#impl-f32--div)
      - [`impl F32 : Eq`](#impl-f32--eq)
      - [`impl F32 : FromBytes`](#impl-f32--frombytes)
      - [`impl F32 : FromString`](#impl-f32--fromstring)
      - [`impl F32 : LessThan`](#impl-f32--lessthan)
      - [`impl F32 : LessThanOrEq`](#impl-f32--lessthanoreq)
      - [`impl F32 : Mul`](#impl-f32--mul)
      - [`impl F32 : Sub`](#impl-f32--sub)
      - [`impl F32 : ToBytes`](#impl-f32--tobytes)
      - [`impl F32 : ToString`](#impl-f32--tostring)
      - [`impl F32 : Zero`](#impl-f32--zero)
    - [F64](#f64)
      - [`abs : F64 -> F64`](#abs--f64---f64)
      - [`to_I8 : F64 -> I8`](#to_i8--f64---i8)
      - [`to_U8 : F64 -> U8`](#to_u8--f64---u8)
      - [`to_I16 : F64 -> I16`](#to_i16--f64---i16)
      - [`to_U16 : F64 -> U16`](#to_u16--f64---u16)
      - [`to_I32 : F64 -> I32`](#to_i32--f64---i32)
      - [`to_U32 : F64 -> U32`](#to_u32--f64---u32)
      - [`to_I64 : F64 -> I64`](#to_i64--f64---i64)
      - [`to_U64 : F64 -> U64`](#to_u64--f64---u64)
      - [`to_F32 : F64 -> F32`](#to_f32--f64---f32)
      - [`to_F64 : F64 -> F64`](#to_f64--f64---f64)
      - [`to_string_exp : F64 -> String`](#to_string_exp--f64---string)
      - [`to_string_exp_precision : U8 -> F64 -> String`](#to_string_exp_precision--u8---f64---string)
      - [`to_string_precision : U8 -> F64 -> String`](#to_string_precision--u8---f64---string)
      - [`impl F64 : Add`](#impl-f64--add)
      - [`impl F64 : Div`](#impl-f64--div)
      - [`impl F64 : Eq`](#impl-f64--eq)
      - [`impl F64 : FromBytes`](#impl-f64--frombytes)
      - [`impl F64 : FromString`](#impl-f64--fromstring)
      - [`impl F64 : LessThan`](#impl-f64--lessthan)
      - [`impl F64 : LessThanOrEq`](#impl-f64--lessthanoreq)
      - [`impl F64 : Mul`](#impl-f64--mul)
      - [`impl F64 : Sub`](#impl-f64--sub)
      - [`impl F64 : ToBytes`](#impl-f64--tobytes)
      - [`impl F64 : ToString`](#impl-f64--tostring)
      - [`impl F64 : Zero`](#impl-f64--zero)
    - [Array](#array)
      - [`@ : I64 -> Array a -> a`](#--i64---array-a---a)
      - [`_get_sub_size_asif : I64 -> I64 -> I64 -> I64 -> Array a -> Array a`](#_get_sub_size_asif--i64---i64---i64---i64---array-a---array-a)
      - [`_unsafe_set_size : I64 -> Array a -> Array a`](#_unsafe_set_size--i64---array-a---array-a)
      - [`_unsafe_get : I64 -> Array a -> a`](#_unsafe_get--i64---array-a---a)
      - [`_unsafe_set : I64 -> a -> Array a -> Array a`](#_unsafe_set--i64---a---array-a---array-a)
      - [`_get_ptr : Array a -> Ptr`](#_get_ptr--array-a---ptr)
      - [`_sort_range_using_buffer : Array a -> I64 -> I64 -> ((a, a) -> Bool) -> Array a -> (Array a, Array a)`](#_sort_range_using_buffer--array-a---i64---i64---a-a---bool---array-a---array-a-array-a)
      - [`act : [f : Functor] I64 -> (a -> f a) -> Array a -> f (Array a)`](#act--f--functor-i64---a---f-a---array-a---f-array-a)
      - [`act! : [f : Functor] I64 -> (a -> f a) -> Array a -> f (Array a)`](#act--f--functor-i64---a---f-a---array-a---f-array-a-1)
      - [`append : Array a -> Array a -> Array a`](#append--array-a---array-a---array-a)
      - [`append! : Array a -> Array a -> Array a`](#append--array-a---array-a---array-a-1)
      - [`borrow_ptr : (Ptr -> b) -> Array a -> b`](#borrow_ptr--ptr---b---array-a---b)
      - [`empty : I64 -> Array a`](#empty--i64---array-a)
      - [`fill : I64 -> a -> Array a`](#fill--i64---a---array-a)
      - [`find_by : [a : Eq] (a -> Bool) -> Array a -> Option I64`](#find_by--a--eq-a---bool---array-a---option-i64)
      - [`force_unique : Array a -> Array a`](#force_unique--array-a---array-a)
      - [`force_unique! : Array a -> Array a`](#force_unique--array-a---array-a-1)
      - [`from_iter : Iterator a -> Array a`](#from_iter--iterator-a---array-a)
      - [`from_map : I64 -> (I64 -> a) -> Array a`](#from_map--i64---i64---a---array-a)
      - [`get_capacity : Array a -> I64`](#get_capacity--array-a---i64)
      - [`get_first : Array a -> Option a`](#get_first--array-a---option-a)
      - [`get_last : Array a -> Option a`](#get_last--array-a---option-a)
      - [`get_size : Array a -> I64`](#get_size--array-a---i64)
      - [`get_sub : I64 -> I64 -> Array a -> Array a`](#get_sub--i64---i64---array-a---array-a)
      - [`is_empty : Array a -> Bool`](#is_empty--array-a---bool)
      - [`mod : I64 -> (a -> a) -> Array a -> Array a`](#mod--i64---a---a---array-a---array-a)
      - [`mod! : I64 -> (a -> a) -> Array a -> Array a`](#mod--i64---a---a---array-a---array-a-1)
      - [`pop_back : Array a -> Array a`](#pop_back--array-a---array-a)
      - [`pop_back! : Array a -> Array a`](#pop_back--array-a---array-a-1)
      - [`push_back : a -> Array a -> Array a`](#push_back--a---array-a---array-a)
      - [`push_back! : a -> Array a -> Array a`](#push_back--a---array-a---array-a-1)
      - [`reserve : I64 -> Array a -> Array a`](#reserve--i64---array-a---array-a)
      - [`set : I64 -> a -> Array a -> Array a`](#set--i64---a---array-a---array-a)
      - [`set! : I64 -> a -> Array a -> Array a`](#set--i64---a---array-a---array-a-1)
      - [`sort_by : ((a, a) -> Bool) -> Array a -> Array a`](#sort_by--a-a---bool---array-a---array-a)
      - [`to_iter : Array a -> Iterator a`](#to_iter--array-a---iterator-a)
      - [`truncate : I64 -> Array a -> Array a`](#truncate--i64---array-a---array-a)
      - [`impl [a : Eq] Array a : Eq`](#impl-a--eq-array-a--eq)
      - [`impl Array : Functor`](#impl-array--functor)
      - [`impl Array : Monad`](#impl-array--monad)
    - [Destructor](#destructor)
      - [`borrow : (a -> b) -> Destructor a -> b`](#borrow--a---b---destructor-a---b)
      - [`make : a -> (a -> ()) -> Destructor a`](#make--a---a------destructor-a)
    - [ErrMsg](#errmsg)
    - [IO](#io)
      - [`_read_line_inner : Bool -> IOHandle -> IOFail ErrMsg String`](#_read_line_inner--bool---iohandle---iofail-errmsg-string)
      - [`_unsafe_perform : IO a -> a`](#_unsafe_perform--io-a---a)
      - [`close_file : IOHandle -> IO ()`](#close_file--iohandle---io-)
      - [`eprint : String -> IO ()`](#eprint--string---io-)
      - [`eprintln : String -> IO ()`](#eprintln--string---io-)
      - [`exit : I64 -> IO a`](#exit--i64---io-a)
      - [`exit_with_msg : I64 -> String -> IO a`](#exit_with_msg--i64---string---io-a)
      - [`input_line : IO String`](#input_line--io-string)
      - [`is_eof : IOHandle -> IO Bool`](#is_eof--iohandle---io-bool)
      - [`loop_lines : IOHandle -> s -> (s -> String -> LoopResult s s) -> IOFail s`](#loop_lines--iohandle---s---s---string---loopresult-s-s---iofail-s)
      - [`loop_lines_io : IOHandle -> s -> (s -> String -> IOFail (LoopResult s s)) -> IOFail s`](#loop_lines_io--iohandle---s---s---string---iofail-loopresult-s-s---iofail-s)
      - [`open_file : Path -> String -> IOFail IOHandle`](#open_file--path---string---iofail-iohandle)
      - [`print : String -> IO ()`](#print--string---io-)
      - [`println : String -> IO ()`](#println--string---io-)
      - [`read_file_string : Path -> IOFail String`](#read_file_string--path---iofail-string)
      - [`read_file_bytes : Path -> IOFail (Array U8)`](#read_file_bytes--path---iofail-array-u8)
      - [`read_line : IOHandle -> IOFail String`](#read_line--iohandle---iofail-string)
      - [`read_bytes : IOHandle -> IOFail (Array U8)`](#read_bytes--iohandle---iofail-array-u8)
      - [`read_n_bytes : IOHandle -> I64 -> IOFail (Array U8)`](#read_n_bytes--iohandle---i64---iofail-array-u8)
      - [`read_string : IOHandle -> IOFail String`](#read_string--iohandle---iofail-string)
      - [`stderr : IOHandle`](#stderr--iohandle)
      - [`stdin : IOHandle`](#stdin--iohandle)
      - [`stdout : IOHandle`](#stdout--iohandle)
      - [`with_file : Path -> String -> (IOHandle -> IOFail a) -> IOFail a`](#with_file--path---string---iohandle---iofail-a---iofail-a)
      - [`write_bytes : IOHandle -> Array U8 -> IOFail ()`](#write_bytes--iohandle---array-u8---iofail-)
      - [`write_file_bytes : Path -> Array U8 -> IOFail ()`](#write_file_bytes--path---array-u8---iofail-)
      - [`write_file_string : Path -> String -> IOFail ()`](#write_file_string--path---string---iofail-)
      - [`write_string : IOHandle -> String -> IOFail ()`](#write_string--iohandle---string---iofail-)
      - [`impl IO : Functor`](#impl-io--functor)
      - [`impl IO : Monad`](#impl-io--monad)
    - [IO::IOHandle](#ioiohandle)
      - [`_file_ptr : IOHandle -> Ptr`.](#_file_ptr--iohandle---ptr)
      - [`_unsafe_close : IOHandle -> ()`](#_unsafe_close--iohandle---)
    - [`from_file_ptr : Ptr -> IOHandle`](#from_file_ptr--ptr---iohandle)
    - [IO::IOFail](#ioiofail)
      - [`from_result : Result ErrMsg a -> IOFail a`](#from_result--result-errmsg-a---iofail-a)
      - [`lift : IO a -> IOFail a`](#lift--io-a---iofail-a)
      - [`throw : ErrMsg -> IOFail a`](#throw--errmsg---iofail-a)
      - [`to_result : IOFail a -> IO (Result ErrMsg a)`](#to_result--iofail-a---io-result-errmsg-a)
      - [`try : (ErrMsg -> IO a) -> IOFail a -> IO a`](#try--errmsg---io-a---iofail-a---io-a)
      - [`impl IOFail : Functor`](#impl-iofail--functor)
      - [`impl IOFail : Monad`](#impl-iofail--monad)
    - [Iterator](#iterator)
      - [`_flatten : Iterator (Iterator a) -> Iterator a`](#_flatten--iterator-iterator-a---iterator-a)
      - [`advance : Iterator a -> Option (a, Iterator a)`](#advance--iterator-a---option-a-iterator-a)
      - [`append : Iterator a -> Iterator a -> Iterator a`](#append--iterator-a---iterator-a---iterator-a)
      - [`bang : Iterator a -> Iterator a`](#bang--iterator-a---iterator-a)
      - [`count_up : I64 -> Iterator I64`](#count_up--i64---iterator-i64)
      - [`empty : Iterator a`](#empty--iterator-a)
      - [`filter : (a -> Bool) -> Iterator a -> Iterator a`](#filter--a---bool---iterator-a---iterator-a)
      - [`find_last : Iterator a -> Option a`](#find_last--iterator-a---option-a)
      - [`fold : b -> (b -> a -> b) -> Iterator a -> b`](#fold--b---b---a---b---iterator-a---b)
      - [`fold_m : [m : Monad] b -> (b -> a -> m b) -> Iterator a -> m b`](#fold_m--m--monad-b---b---a---m-b---iterator-a---m-b)
      - [`from_array : Array a -> Iterator a`](#from_array--array-a---iterator-a)
      - [`from_map : (I64 -> a) -> Iterator a`](#from_map--i64---a---iterator-a)
      - [`generate : s -> (s -> Option (a, s)) -> Iterator a`](#generate--s---s---option-a-s---iterator-a)
      - [`get_first : Iterator a -> Option a`](#get_first--iterator-a---option-a)
      - [`get_size : Iterator a -> I64`](#get_size--iterator-a---i64)
      - [`get_tail : Iterator a -> Option (Iterator a)`](#get_tail--iterator-a---option-iterator-a)
      - [`intersperse : a -> Iterator a -> Iterator a`](#intersperse--a---iterator-a---iterator-a)
      - [`is_empty : Iterator a -> Bool`](#is_empty--iterator-a---bool)
      - [`loop_iter : b -> (b -> a -> LoopResult b b) -> Iterator a -> b`](#loop_iter--b---b---a---loopresult-b-b---iterator-a---b)
      - [`loop_iter_m : [m : Monad] b -> (b -> a -> m (LoopResult b b)) -> Iterator a -> m b`](#loop_iter_m--m--monad-b---b---a---m-loopresult-b-b---iterator-a---m-b)
      - [`push_front : a -> Iterator a -> Iterator a`](#push_front--a---iterator-a---iterator-a)
      - [`range : I64 -> I64 -> Iterator I64`](#range--i64---i64---iterator-i64)
      - [`reverse : Iterator a -> Iterator a`](#reverse--iterator-a---iterator-a)
      - [`subsequences : Iterator a -> Iterator (Iterator a)`](#subsequences--iterator-a---iterator-iterator-a)
      - [`sum : [a : Additive] Iterator a -> a`](#sum--a--additive-iterator-a---a)
      - [`take : I64 -> Iterator a -> Iterator a`](#take--i64---iterator-a---iterator-a)
      - [`take_while : (a -> Bool) -> Iterator a -> Iterator a`](#take_while--a---bool---iterator-a---iterator-a)
      - [`to_array : Iterator a -> Array a`](#to_array--iterator-a---array-a)
      - [`zip : Iterator b -> Iterator a -> Iterator (a, b)`](#zip--iterator-b---iterator-a---iterator-a-b)
      - [`impl Iterator a : Add`](#impl-iterator-a--add)
      - [`impl [a : Eq] Iterator a : Eq`](#impl-a--eq-iterator-a--eq)
      - [`impl Iterator : Functor`](#impl-iterator--functor)
      - [`impl Iterator : Monad`](#impl-iterator--monad)
    - [Lazy](#lazy)
    - [LoopResult](#loopresult)
      - [`break_m : [m : Monad] r -> m (LoopResult s r)`](#break_m--m--monad-r---m-loopresult-s-r)
      - [`continue_m : [m : Monad] s -> m (LoopResult s r)`](#continue_m--m--monad-s---m-loopresult-s-r)
    - [Option](#option)
      - [`as_some_or : a -> Option a -> a`](#as_some_or--a---option-a---a)
      - [`map_or : b -> (a -> b) -> Option a -> b`](#map_or--b---a---b---option-a---b)
      - [`impl [a : Eq] Option a : Eq`](#impl-a--eq-option-a--eq)
      - [`impl Option : Functor`](#impl-option--functor)
      - [`impl Option : Monad`](#impl-option--monad)
    - [Path](#path)
      - [`parse : String -> Option Path`](#parse--string---option-path)
      - [`impl Path : ToString`](#impl-path--tostring)
    - [Ptr](#ptr)
      - [`add_offset : I64 -> Ptr -> Ptr`](#add_offset--i64---ptr---ptr)
      - [`subtract_ptr : Ptr -> Ptr -> I64`](#subtract_ptr--ptr---ptr---i64)
      - [`impl Ptr : Eq`](#impl-ptr--eq)
      - [`impl Ptr : ToString`](#impl-ptr--tostring)
    - [PunchedArray](#punchedarray)
      - [`plug_in! : a -> PunchedArray a -> Array a`](#plug_in--a---punchedarray-a---array-a)
      - [`punch! : I64 -> Array a -> (PunchedArray a, a)`](#punch--i64---array-a---punchedarray-a-a)
    - [Result](#result)
      - [`unwrap : Result e o -> o`](#unwrap--result-e-o---o)
      - [`impl Result e : Monad`](#impl-result-e--monad)
    - [String](#string)
      - [`_unsafe_from_c_str : Array U8 -> String`](#_unsafe_from_c_str--array-u8---string)
      - [`_unsafe_from_c_str_ptr : Ptr -> String`](#_unsafe_from_c_str_ptr--ptr---string)
      - [`_get_c_str : String -> Ptr`](#_get_c_str--string---ptr)
      - [`borrow_c_str : (Ptr -> a) -> String -> a`](#borrow_c_str--ptr---a---string---a)
      - [`concat : String -> String -> String`](#concat--string---string---string)
      - [`concat_iter : Iterator String -> String`](#concat_iter--iterator-string---string)
      - [`empty : I64 -> String`](#empty--i64---string)
      - [`find : String -> I64 -> String -> Option I64`](#find--string---i64---string---option-i64)
      - [`get_bytes : String -> Array U8`](#get_bytes--string---array-u8)
      - [`get_first_byte : String -> Option Byte`](#get_first_byte--string---option-byte)
      - [`get_last_byte : String -> Option Byte`](#get_last_byte--string---option-byte)
      - [`get_size : String -> I64`](#get_size--string---i64)
      - [`get_sub : I64 -> I64 -> String -> String`](#get_sub--i64---i64---string---string)
      - [`is_empty : String -> Bool`](#is_empty--string---bool)
      - [`join : String -> Iterator String -> String`](#join--string---iterator-string---string)
      - [`pop_back_byte : String -> String`](#pop_back_byte--string---string)
      - [`split : String -> String -> Iterator String`](#split--string---string---iterator-string)
      - [`strip_first_bytes : (U8 -> Bool) -> String -> String`](#strip_first_bytes--u8---bool---string---string)
      - [`strip_first_spaces : String -> String`](#strip_first_spaces--string---string)
      - [`strip_last_bytes : (U8 -> Bool) -> String -> String`](#strip_last_bytes--u8---bool---string---string)
      - [`strip_last_newlines : String -> String`](#strip_last_newlines--string---string)
      - [`strip_last_spaces : String -> String`](#strip_last_spaces--string---string)
      - [`strip_spaces : String -> String`](#strip_spaces--string---string)
      - [`impl String : Add`](#impl-string--add)
      - [`impl String : Eq`](#impl-string--eq)
      - [`impl String : ToString`](#impl-string--tostring)
    - [Tuple{N}](#tuplen)
      - [`impl [a : Eq, b : Eq] (a, b) : Eq`](#impl-a--eq-b--eq-a-b--eq)
      - [`impl [a : ToString, b : ToString] (a, b) : ToString`](#impl-a--tostring-b--tostring-a-b--tostring)
    - [`()`](#)
      - [`impl () : Eq`](#impl---eq)
  - [Functions](#functions)
    - [`abort : Lazy a`](#abort--lazy-a)
    - [`compose : (a -> b) -> (b -> c) -> a -> c`](#compose--a---b---b---c---a---c)
    - [`fix : ((a -> b) -> a -> b) -> a -> b`](#fix--a---b---a---b---a---b)
    - [`loop : s -> (s -> LoopResult s r) -> r`](#loop--s---s---loopresult-s-r---r)
    - [`loop_m : [m : Monad] s -> (s -> m (LoopResult s r)) -> m r`](#loop_m--m--monad-s---s---m-loopresult-s-r---m-r)
    - [`unsafe_is_unique : a -> (Bool, a)`](#unsafe_is_unique--a---bool-a)
  - [Traits](#traits)
    - [Additive](#additive)
    - [FromBytes](#frombytes)
      - [`from_bytes : [a : FromBytes] Array U8 -> Result ErrMsg a`](#from_bytes--a--frombytes-array-u8---result-errmsg-a)
    - [FromString](#fromstring)
      - [`from_string : [a : FromString] String -> Result ErrMsg a`](#from_string--a--fromstring-string---result-errmsg-a)
    - [Functor (\* -\> \*)](#functor----)
      - [`map : [f : Functor] (a -> b) -> f a -> f b`](#map--f--functor-a---b---f-a---f-b)
    - [LessThan](#lessthan)
      - [`less_than : [a : LessThan] a -> a -> a`](#less_than--a--lessthan-a---a---a)
      - [`max : [a : LessThan] a -> a -> a`](#max--a--lessthan-a---a---a)
      - [`min : [a : LessThan] a -> a -> a`](#min--a--lessthan-a---a---a)
    - [LessThanOrEq](#lessthanoreq)
      - [`less_than_or_eq : [a : LessThanOrEq] a -> a -> a`](#less_than_or_eq--a--lessthanoreq-a---a---a)
    - [Monad (\* -\> \*)](#monad----)
      - [(required) `bind : [m : Monad] (a -> m b) -> m a -> m b`](#required-bind--m--monad-a---m-b---m-a---m-b)
      - [(required) `pure : [m : Monad] a -> m a`](#required-pure--m--monad-a---m-a)
      - [`flatten : [m : Monad] m (m a) -> a`](#flatten--m--monad-m-m-a---a)
    - [ToBytes](#tobytes)
      - [`to_bytes : [a : ToBytes] a -> Array U8`](#to_bytes--a--tobytes-a---array-u8)
    - [ToString](#tostring)
      - [`to_string : [a: ToString] a -> String`](#to_string--a-tostring-a---string)
- [Module `Character`](#module-character)
  - [`is_alnum : U8 -> Bool`](#is_alnum--u8---bool)
  - [`is_alpha : U8 -> Bool`](#is_alpha--u8---bool)
  - [`is_blank : U8 -> Bool`](#is_blank--u8---bool)
  - [`is_cntrl : U8 -> Bool`](#is_cntrl--u8---bool)
  - [`is_digit : U8 -> Bool`](#is_digit--u8---bool)
  - [`is_graph : U8 -> Bool`](#is_graph--u8---bool)
  - [`is_lower : U8 -> Bool`](#is_lower--u8---bool)
  - [`is_print : U8 -> Bool`](#is_print--u8---bool)
  - [`is_punct : U8 -> Bool`](#is_punct--u8---bool)
  - [`is_space : U8 -> Bool`](#is_space--u8---bool)
  - [`is_upper : U8 -> Bool`](#is_upper--u8---bool)
  - [`is_xdigit : U8 -> Bool`](#is_xdigit--u8---bool)
  - [`to_lower : U8 -> U8`](#to_lower--u8---u8)
  - [`to_upper : U8 -> U8`](#to_upper--u8---u8)
- [Module `Debug`](#module-debug)
  - [`_debug_print_to_stream : IOHandle -> String -> ()`](#_debug_print_to_stream--iohandle---string---)
  - [`assert : Lazy String -> Bool -> ()`](#assert--lazy-string---bool---)
  - [`assert_eq : [a: Eq] Lazy String -> a -> a -> ()`](#assert_eq--a-eq-lazy-string---a---a---)
  - [`assert_unique! : Lazy String -> a -> a`](#assert_unique--lazy-string---a---a)
  - [`consumed_time_while : (a -> b) -> a -> (b, F64)`](#consumed_time_while--a---b---a---b-f64)
  - [`consumed_time_while_io : IO a -> IO (a, F64)`](#consumed_time_while_io--io-a---io-a-f64)
  - [`consumed_time_while_lazy : Lazy a -> (a, F64)`](#consumed_time_while_lazy--lazy-a---a-f64)
  - [`debug_eprint : String -> ()`](#debug_eprint--string---)
  - [`debug_eprintln : String -> ()`](#debug_eprintln--string---)
  - [`debug_print : String -> ()`](#debug_print--string---)
  - [`debug_println : String -> ()`](#debug_println--string---)
- [Module `Hash`](#module-hash)
  - [`trait a : Hash`](#trait-a--hash)
  - [`trait HashKey = Hash + Eq`](#trait-hashkey--hash--eq)
    - [(required) `hash : [a : Hash] a -> U64`](#required-hash--a--hash-a---u64)
  - [`impl [a : Hash] Array a : Hash`](#impl-a--hash-array-a--hash)
  - [`impl String : Hash`](#impl-string--hash)
  - [`impl U8 : Hash`](#impl-u8--hash)
  - [`impl I64 : Hash`](#impl-i64--hash)
  - [`impl U64 : Hash`](#impl-u64--hash)
  - [`impl [a : Hash, b : Hash] (a, b) : Hash`](#impl-a--hash-b--hash-a-b--hash)
- [Module `HashMap`](#module-hashmap)
  - [`type HashMap k v`](#type-hashmap-k-v)
  - [`_find_place : [k : HashKey] k -> HashMap k v -> (I64, Option I64)`](#_find_place--k--hashkey-k---hashmap-k-v---i64-option-i64)
  - [`_get_pot_geq : I64 -> I64`](#_get_pot_geq--i64---i64)
  - [`contains_key : [k : HashKey] k -> HashMap k v -> Bool`](#contains_key--k--hashkey-k---hashmap-k-v---bool)
  - [`empty : I64 -> HashMap k v`](#empty--i64---hashmap-k-v)
  - [`erase : [k : HashKey] k -> HashMap k v -> HashMap k v`](#erase--k--hashkey-k---hashmap-k-v---hashmap-k-v)
  - [`find : [k : HashKey] k -> HashMap k v -> Option v`](#find--k--hashkey-k---hashmap-k-v---option-v)
  - [`find_or : [k : HashKey] k -> v -> HashMap k v -> Option v`](#find_or--k--hashkey-k---v---hashmap-k-v---option-v)
  - [`get_capacity : HashMap k v -> I64`](#get_capacity--hashmap-k-v---i64)
  - [`get_size : HashMap k v -> I64`](#get_size--hashmap-k-v---i64)
  - [`insert : [k : HashKey] k -> v -> HashMap k v -> HashMap k v`](#insert--k--hashkey-k---v---hashmap-k-v---hashmap-k-v)
  - [`reserve : [k : HashKey] I64 -> HashMap k v -> HashMap k v`](#reserve--k--hashkey-i64---hashmap-k-v---hashmap-k-v)
  - [`to_iter : HashMap k v -> Iterator (k, v)`](#to_iter--hashmap-k-v---iterator-k-v)
- [Module `HashSet`](#module-hashset)
  - [`type HashSet k`](#type-hashset-k)
  - [`contains : [k : HashKey] k -> HashSet k -> Bool`](#contains--k--hashkey-k---hashset-k---bool)
  - [`empty : I64 -> HashSet k`](#empty--i64---hashset-k)
  - [`erase : [k : HashKey] k -> HashSet k -> HashSet k`](#erase--k--hashkey-k---hashset-k---hashset-k)
  - [`from_iter : [k : HashKey] Iterator k -> HashSet k`](#from_iter--k--hashkey-iterator-k---hashset-k)
  - [`get_capacity : HashSet k -> I64`](#get_capacity--hashset-k---i64)
  - [`get_size : HashSet k -> I64`](#get_size--hashset-k---i64)
  - [`insert : [k : HashKey] k -> HashSet k -> HashSet k`](#insert--k--hashkey-k---hashset-k---hashset-k)
  - [`intersect : [k : HashKey] HashSet k -> HashSet k -> HashSet k`](#intersect--k--hashkey-hashset-k---hashset-k---hashset-k)
  - [`merge : [k : HashKey] HashSet k -> HashSet k -> HashSet k`](#merge--k--hashkey-hashset-k---hashset-k---hashset-k)
  - [`reserve : [k : HashKey] I64 -> HashSet k -> HashSet k`](#reserve--k--hashkey-i64---hashset-k---hashset-k)
  - [`to_iter : HashSet k -> Iterator k`](#to_iter--hashset-k---iterator-k)
- [Module `Math`](#module-math)
  - [`_gcd_nonneg : I64 -> I64 -> I64`](#_gcd_nonneg--i64---i64---i64)
  - [`acos : F64 -> F64`](#acos--f64---f64)
  - [`asin : F64 -> F64`](#asin--f64---f64)
  - [`atan : F64 -> F64`](#atan--f64---f64)
  - [`atan2 : F64 -> F64 -> F64`](#atan2--f64---f64---f64)
  - [`binomial_coefficients : I64 -> Array (Array I64)`](#binomial_coefficients--i64---array-array-i64)
  - [`ceil : F64 -> F64`](#ceil--f64---f64)
  - [`cos : F64 -> F64`](#cos--f64---f64)
  - [`cosh : F64 -> F64`](#cosh--f64---f64)
  - [`exp : F64 -> F64`](#exp--f64---f64)
  - [`floor : F64 -> F64`](#floor--f64---f64)
  - [`fmod : F64 -> F64 -> F64`](#fmod--f64---f64---f64)
  - [`frexp : F64 -> (F64, I32)`](#frexp--f64---f64-i32)
  - [`gcd : I64 -> I64 -> I64`](#gcd--i64---i64---i64)
  - [`ldexp : I32 -> F64 -> F64`](#ldexp--i32---f64---f64)
  - [`log : F64 -> F64`](#log--f64---f64)
  - [`log10 : F64 -> F64`](#log10--f64---f64)
  - [`modf : F64 -> (F64, F64)`](#modf--f64---f64-f64)
  - [`pi32 : F32`](#pi32--f32)
  - [`pi64 : F64`](#pi64--f64)
  - [`pow : F64 -> F64 -> F64`](#pow--f64---f64---f64)
  - [`sin : F64 -> F64`](#sin--f64---f64)
  - [`sinh : F64 -> F64`](#sinh--f64---f64)
  - [`sqrt : F64 -> F64`](#sqrt--f64---f64)
  - [`tan : F64 -> F64`](#tan--f64---f64)
  - [`tanh : F64 -> F64`](#tanh--f64---f64)
- [module `Subprocess`](#module-subprocess)
  - [`type ExitStatus`](#type-exitstatus)
  - [`run_string : String -> Array String -> String -> IOFail ((String, String), ExitStatus)`](#run_string--string---array-string---string---iofail-string-string-exitstatus)
  - [`run_with_stream : String -> Array String -> ((IOHandle, IOHandle, IOHandle) -> IOFail a) -> IOFail (a, ExitStatus)`](#run_with_stream--string---array-string---iohandle-iohandle-iohandle---iofail-a---iofail-a-exitstatus)
- [module `Time`](#module-time)
  - [`type Time`](#type-time)
  - [`type DateTime`](#type-datetime)
  - [`_datetime_to_time_inner : Bool -> DateTime -> Result ErrMsg Time`](#_datetime_to_time_inner--bool---datetime---result-errmsg-time)
  - [`_time_to_datetime_inner : Bool -> Time -> Result ErrMsg DateTime`](#_time_to_datetime_inner--bool---time---result-errmsg-datetime)
  - [`from_local : DateTime -> IOResult ErrMsg Time`](#from_local--datetime---ioresult-errmsg-time)
  - [`from_utc : DateTime -> Result ErrMsg Time`](#from_utc--datetime---result-errmsg-time)
  - [`get_now : IO Time`](#get_now--io-time)
  - [`to_F64 : Time -> F64`](#to_f64--time---f64)
  - [`to_local : Time -> IOResult ErrMsg DateTime`](#to_local--time---ioresult-errmsg-datetime)
  - [`to_utc : Time -> Result ErrMsg DateTime`](#to_utc--time---result-errmsg-datetime)

# NOTE

Built-in libraries of Fix is currently growing, and destructive changes are made frequently.

# Module `Std`

`Std` is a module which is implicitly imported so you don't need to write `import Std`.

## Types, related values and implementations

### Bool

`Bool` is the type of boolean values, represented by 8-bit integer `1` (`true`) and `0` (`false`). 

Boolean literals are `true` and `false`.

#### `impl Bool : Eq`
#### `impl Bool : ToString`

### I8

`I8` is the type of 8-bit unsigned integers.

Literals:

- `{number}_I8`
    - Example: `42_I8`

#### `abs : I8 -> I8`
#### `maximum : I8`
#### `minimum : I8`
#### `bit_and : I8 -> I8 -> I8`
#### `bit_or : I8 -> I8 -> I8`
#### `bit_xor : I8 -> I8 -> I8`
#### `shift_left : I8 -> I8 -> I8`
#### `shift_right : I8 -> I8 -> I8`
#### `to_I8 : I8 -> I8`
#### `to_U8 : I8 -> U8`
#### `to_I16 : I8 -> I16`
#### `to_U16 : I8 -> U16`
#### `to_I32 : I8 -> I32`
#### `to_U32 : I8 -> U32`
#### `to_I64 : I8 -> I64`
#### `to_U64 : I8 -> U64`
#### `to_F32 : I8 -> F32`
#### `to_F64 : I8 -> F64`
#### `impl I8 : Add`
#### `impl I8 : Eq`
#### `impl I8 : FromBytes`
#### `impl I8 : FromString`
#### `impl I8 : LessThan`
#### `impl I8 : LessThanOrEq`
#### `impl I8 : Mul`
#### `impl I8 : Neg`
#### `impl I8 : Rem`
#### `impl I8 : Sub`
#### `impl I8 : ToBytes`
#### `impl I8 : ToString`
#### `impl I8 : Zero`

### U8

`U8` is the type of 8-bit unsigned integers.

Literals:

- `{number}_U8`
    - Example: `-1_U8 == 255_U8`
- `'{character}'`
  - Example: 
    - `'A'` for `65_U8`
    - `'\0'` for `0_U8`
    - `'\t'` for `9_U8`
    - `'\r'` for `13_U8`
    - `'\n'` for `10_U8`
    - `'\\'` for `92_U8`
    - `'\''` for `39_U8`
    - `'\x7f'` for `127_U8`

#### `maximum : U8`
#### `minimum : U8`
#### `bit_and : U8 -> U8 -> U8`
#### `bit_or : U8 -> U8 -> U8`
#### `bit_xor : U8 -> U8 -> U8`
#### `shift_left : U8 -> U8 -> U8`
#### `shift_right : U8 -> U8 -> U8`
#### `to_I8 : U8 -> I8`
#### `to_U8 : U8 -> U8`
#### `to_I16 : U8 -> I16`
#### `to_U16 : U8 -> U16`
#### `to_I32 : U8 -> I32`
#### `to_U32 : U8 -> U32`
#### `to_I64 : U8 -> I64`
#### `to_U64 : U8 -> U64`
#### `to_F32 : U8 -> F32`
#### `to_F64 : U8 -> F64`
#### `impl U8 : Add`
#### `impl U8 : Eq`
#### `impl U8 : FromBytes`
#### `impl U8 : FromString`
#### `impl U8 : LessThan`
#### `impl U8 : LessThanOrEq`
#### `impl U8 : Mul`
#### `impl U8 : Neg`
#### `impl U8 : Rem`
#### `impl U8 : Sub`
#### `impl U8 : ToBytes`
#### `impl U8 : ToString`
#### `impl U8 : Zero`

### I16

`I16` is the type of 8-bit unsigned integers.

Literals:

- `{number}_I16`
    - Example: `42_I16`

#### `abs : I16 -> I16`
#### `maximum : I16`
#### `minimum : I16`
#### `bit_and : I16 -> I16 -> I16`
#### `bit_or : I16 -> I16 -> I16`
#### `bit_xor : I16 -> I16 -> I16`
#### `shift_left : I16 -> I16 -> I16`
#### `shift_right : I16 -> I16 -> I16`
#### `to_I8 : I16 -> I8`
#### `to_U8 : I16 -> U8`
#### `to_I16 : I16 -> I16`
#### `to_U16 : I16 -> U16`
#### `to_I32 : I16 -> I32`
#### `to_U32 : I16 -> U32`
#### `to_I64 : I16 -> I64`
#### `to_U64 : I16 -> U64`
#### `to_F32 : I16 -> F32`
#### `to_F64 : I16 -> F64`
#### `impl I16 : Add`
#### `impl I16 : Eq`
#### `impl I16 : FromBytes`
#### `impl I16 : FromString`
#### `impl I16 : LessThan`
#### `impl I16 : LessThanOrEq`
#### `impl I16 : Mul`
#### `impl I16 : Neg`
#### `impl I16 : Rem`
#### `impl I16 : Sub`
#### `impl I16 : ToBytes`
#### `impl I16 : ToString`
#### `impl I16 : Zero`

### U16

`U16` is the type of 8-bit unsigned integers.

Literals:

- `{number}_U16`
    - Example: `42_U16`

#### `maximum : U16`
#### `minimum : U16`
#### `bit_and : U16 -> U16 -> U16`
#### `bit_or : U16 -> U16 -> U16`
#### `bit_xor : U16 -> U16 -> U16`
#### `shift_left : U16 -> U16 -> U16`
#### `shift_right : U16 -> U16 -> U16`
#### `to_I8 : U16 -> I8`
#### `to_U8 : U16 -> U8`
#### `to_I16 : U16 -> I16`
#### `to_U16 : U16 -> U16`
#### `to_I32 : U16 -> I32`
#### `to_U32 : U16 -> U32`
#### `to_I64 : U16 -> I64`
#### `to_U64 : U16 -> U64`
#### `to_F32 : U16 -> F32`
#### `to_F64 : U16 -> F64`
#### `impl U16 : Add`
#### `impl U16 : Eq`
#### `impl U16 : FromBytes`
#### `impl U16 : FromString`
#### `impl U16 : LessThan`
#### `impl U16 : LessThanOrEq`
#### `impl U16 : Mul`
#### `impl U16 : Neg`
#### `impl U16 : Rem`
#### `impl U16 : Sub`
#### `impl U16 : ToBytes`
#### `impl U16 : ToString`
#### `impl U16 : Zero`


### I32

`I32` is the type of 32-bit signed integers.

Literals:
- `{number}_I32`
    - Example: `42_I32`

#### `abs : I32 -> I32`
#### `maximum : I32`
#### `minimum : I32`
#### `bit_and : I32 -> I32 -> I32`
#### `bit_or : I32 -> I32 -> I32`
#### `bit_xor : I32 -> I32 -> I32`
#### `shift_left : I32 -> I32 -> I32`
#### `shift_right : I32 -> I32 -> I32`
#### `to_I8 : I32 -> I8`
#### `to_U8 : I32 -> U8`
#### `to_I16 : I32 -> I16`
#### `to_U16 : I32 -> U16`
#### `to_U32 : I32 -> U32`
#### `to_I64 : I32 -> I64`
#### `to_U64 : I32 -> U64`
#### `to_F32 : I32 -> F32`
#### `to_F64 : I32 -> F64`
#### `impl I32 : Add`
#### `impl I32 : Eq`
#### `impl I32 : FromBytes`
#### `impl I32 : FromString`
#### `impl I32 : LessThan`
#### `impl I32 : LessThanOrEq`
#### `impl I32 : Mul`
#### `impl I32 : Neg`
#### `impl I32 : Rem`
#### `impl I32 : Sub`
#### `impl I32 : ToBytes`
#### `impl I32 : ToString`
#### `impl I32 : Zero`

### U32

`U32` is the type of 32-bit unsigned integers.

Literals:

- `{number}_U32`
    - Example: `-1_U32 == 4294967295_U32`

#### `maximum : U32`
#### `minimum : U32`
#### `bit_and : U32 -> U32 -> U32`
#### `bit_or : U32 -> U32 -> U32`
#### `bit_xor : U32 -> U32 -> U32`
#### `shift_left : U32 -> U32 -> U32`
#### `shift_right : U32 -> U32 -> U32`
#### `to_I8 : U32 -> I8`
#### `to_U8 : U32 -> U8`
#### `to_I16 : U32 -> I16`
#### `to_U16 : U32 -> U16`
#### `to_I32 : U32 -> I32`
#### `to_I64 : U32 -> I64`
#### `to_U64 : U32 -> U64`
#### `to_F32 : U32 -> F32`
#### `to_F64 : U32 -> F64`
#### `impl U32 : Add`
#### `impl U32 : Eq`
#### `impl U32 : FromBytes`
#### `impl U32 : FromString`
#### `impl U32 : LessThan`
#### `impl U32 : LessThanOrEq`
#### `impl U32 : Mul`
#### `impl U32 : Neg`
#### `impl U32 : Rem`
#### `impl U32 : Sub`
#### `impl U32 : ToBytes`
#### `impl U32 : ToString`
#### `impl U32 : Zero`

### I64

`I64` is the type of 64-bit signed integers.

Literals:
- `{number}`
    - Example: `42`
- `{number}_I64`
    - Example: `42_I64 == 42`

#### `abs : I64 -> I64`
#### `maximum : I64`
#### `minimum : I64`
#### `bit_and : I64 -> I64 -> I64`
#### `bit_or : I64 -> I64 -> I64`
#### `bit_xor : I64 -> I64 -> I64`
#### `shift_left : I64 -> I64 -> I64`
#### `shift_right : I64 -> I64 -> I64`
#### `to_I8 : I64 -> I8`
#### `to_U8 : I64 -> U8`
#### `to_I16 : I64 -> I16`
#### `to_U16 : I64 -> U16`
#### `to_I32 : I64 -> I32`
#### `to_U32 : I64 -> U32`
#### `to_U64 : I64 -> U64`
#### `to_F32 : I64 -> F32`
#### `to_F64 : I64 -> F64`
#### `impl I64 : Add`
#### `impl I64 : Eq`
#### `impl I64 : FromBytes`
#### `impl I64 : FromString`
#### `impl I64 : LessThan`
#### `impl I64 : LessThanOrEq`
#### `impl I64 : Mul`
#### `impl I64 : Neg`
#### `impl I64 : Rem`
#### `impl I64 : Sub`
#### `impl I64 : ToBytes`
#### `impl I64 : ToString`
#### `impl I64 : Zero`

### U64

`U64` is the type of 64-bit unsigned integers.

Literals:

- `{number}_U64`
    - Example: `-1_U64 == 18446744073709551615_U64`

#### `maximum : U64`
#### `minimum : U64`
#### `bit_and : U64 -> U64 -> U64`
#### `bit_or : U64 -> U64 -> U64`
#### `bit_xor : U64 -> U64 -> U64`
#### `shift_left : U64 -> U64 -> U64`
#### `shift_right : U64 -> U64 -> U64`
#### `to_I8 : U64 -> I8`
#### `to_U8 : U64 -> U8`
#### `to_I16 : U64 -> I16`
#### `to_U16 : U64 -> U16`
#### `to_I32 : U64 -> I32`
#### `to_U32 : U64 -> U32`
#### `to_I64 : U64 -> I64`
#### `to_F32 : U64 -> F32`
#### `to_F64 : U64 -> F64`
#### `impl U64 : Add`
#### `impl U64 : Eq`
#### `impl U64 : FromBytes`
#### `impl U64 : FromString`
#### `impl U64 : LessThan`
#### `impl U64 : LessThanOrEq`
#### `impl U64 : Mul`
#### `impl U64 : Neg`
#### `impl U64 : Rem`
#### `impl U64 : Sub`
#### `impl U64 : ToBytes`
#### `impl U64 : ToString`
#### `impl U64 : Zero`

### F32

`F32` is the type of 32-bit floating numbers.

For `F32` literals, you need to add a suffix "_F32" to explicitly specify the type. Example: `3.1416_F32`.

#### `abs : F32 -> F32`
#### `to_I8 : F32 -> I8`
#### `to_U8 : F32 -> U8`
#### `to_I16 : F32 -> I16`
#### `to_U16 : F32 -> U16`
#### `to_I32 : F32 -> I32`
#### `to_U32 : F32 -> U32`
#### `to_I64 : F32 -> I64`
#### `to_U64 : F32 -> U64`
#### `to_F32 : F32 -> F32`
#### `to_F64 : F32 -> F64`
#### `to_string_exp : F32 -> String`
Convert a floating number to a string of exponential form.

#### `to_string_exp_precision : U8 -> F32 -> String`
Convert a floating number to a string of exponential form with specified precision (i.e., number of digits after the decimal point).

#### `to_string_precision : U8 -> F32 -> String`
Convert a floating number to a string with specified precision (i.e., number of digits after the decimal point).

#### `impl F32 : Add`
#### `impl F32 : Div`
#### `impl F32 : Eq`
#### `impl F32 : FromBytes`
#### `impl F32 : FromString`
#### `impl F32 : LessThan`
#### `impl F32 : LessThanOrEq`
#### `impl F32 : Mul`
#### `impl F32 : Sub`
#### `impl F32 : ToBytes`
#### `impl F32 : ToString`
#### `impl F32 : Zero`

### F64

`F64` is the type of 64-bit floating numbers.

For `F64` literals, you can write or omit explicit type specifier suffix "_F64". Example `3.1416_F64 == 3.1416`.

#### `abs : F64 -> F64`
#### `to_I8 : F64 -> I8`
#### `to_U8 : F64 -> U8`
#### `to_I16 : F64 -> I16`
#### `to_U16 : F64 -> U16`
#### `to_I32 : F64 -> I32`
#### `to_U32 : F64 -> U32`
#### `to_I64 : F64 -> I64`
#### `to_U64 : F64 -> U64`
#### `to_F32 : F64 -> F32`
#### `to_F64 : F64 -> F64`
#### `to_string_exp : F64 -> String`
Convert a floating number to a string of exponential form.

#### `to_string_exp_precision : U8 -> F64 -> String`
Convert a floating number to a string of exponential form with specified precision (i.e., number of digits after the decimal point).

#### `to_string_precision : U8 -> F64 -> String`
Convert a floating number to a string with specified precision (i.e., number of digits after the decimal point).

#### `impl F64 : Add`
#### `impl F64 : Div`
#### `impl F64 : Eq`
#### `impl F64 : FromBytes`
#### `impl F64 : FromString`
#### `impl F64 : LessThan`
#### `impl F64 : LessThanOrEq`
#### `impl F64 : Mul`
#### `impl F64 : Sub`
#### `impl F64 : ToBytes`
#### `impl F64 : ToString`
#### `impl F64 : Zero`

### Array

`Array` is the type of variable-length arrays.

Literals: 
- `[{elem_0}, {elem_1}, ...]`
    - Example: `[1, 2, 3]` for integer array of length 3.

Methods:

#### `@ : I64 -> Array a -> a`
Returns an element of an array at an index.

#### `_get_sub_size_asif : I64 -> I64 -> I64 -> I64 -> Array a -> Array a`
A function like `get_sub`, but behaves as if the size of the array is the specified value,
and has a parameter to specify additional capacity of the returned `Array`.

#### `_unsafe_set_size : I64 -> Array a -> Array a`
Updates the length of an array, without uniqueness checking or validation of the given length value.

#### `_unsafe_get : I64 -> Array a -> a`
Gets a value from an array, without bounds checking and retaining the returned value.

#### `_unsafe_set : I64 -> a -> Array a -> Array a`
Sets a value into an array, without uniqueness checking, bounds checking and releasing the old value.

#### `_get_ptr : Array a -> Ptr`
Get the pointer to the memory region where elements are stored.
Note that in case the array is not used after call of this function, the returned pointer will be already released.

#### `_sort_range_using_buffer : Array a -> I64 -> I64 -> ((a, a) -> Bool) -> Array a -> (Array a, Array a)`
Sort elements in a range of an array by "less than" comparator.
This function receives a working buffer as the first argument to reduce memory allocation, and returns it as second element.

#### `act : [f : Functor] I64 -> (a -> f a) -> Array a -> f (Array a)`
Functorial version of `Array::mod`, a.k.a. "lens" in Haskell community.

This function can be defined for any functor `f` in general, but it is easier to understand the behavior when `f` is a monad: the monadic action `act(idx, fun, arr)` first performs `fun(arr.@(idx))` to get a value `elm`, and returns a pure value `arr.set(idx, elm)`. In short, this function modifies an array by a monadic action. 

This action can be implemented as `fun(arr.@(idx)).bind(|elm| pure $ arr.set(idx, elm))`. As we have identity `map(f) == bind(|x| pure $ f(x))` for `map` of a functor underlying a monad, it can be written as `fun(arr.@(idx)).map(|elm| arr.set(idx, elm))` and therefore this function can be defined using only a method of a functor.

What is special about this function is that if you call `arr.act(idx, fun)` when both of `arr` and `arr.@(idx)` are unique, it is assured that `fun` receives the element which is unique.

If you call `act` on an array which is shared, this function clones the given array when inserting the result of your action into the array. This means that you don't need to pay cloning cost when your action failed, as expected.

#### `act! : [f : Functor] I64 -> (a -> f a) -> Array a -> f (Array a)`
This function is almost the same as `Array::act`, but it panics if the given array is shared.

#### `append : Array a -> Array a -> Array a`
Append an array to an array.
Note 1: Since `a1.append(a2)` puts `a2` after `a1`, `append(lhs, rhs)` puts `lhs` after `rhs`.
Note 2: 
As an optimization, when `a1` is empty, `a1.append(a2)` may return `a2` itself.
So even if you call `append` on an unique empty array, the returned array can be a shared one.

#### `append! : Array a -> Array a -> Array a`
Append an array to an array.
This is similar to `Array::append`, but `a1.append!(a2)` panics if this function has to clone `a1` due to it being shared.
Note that, when the capacity of `a1` is less than `a1.get_size + a2.get_size`, then `a1.append!(a2)` will not panic even if `a1` is shared, because in this case cloning is inevitable whether or not `a1` is shared.

#### `borrow_ptr : (Ptr -> b) -> Array a -> b`
Call a function with a pointer to the memory region where elements are stored.

#### `empty : I64 -> Array a`
Creates an empty array with specified capacity.

#### `fill : I64 -> a -> Array a`
Creates an array filled with the initial value.
The capacity is set to the same value as the length.
Example: `fill(n, x) == [x, x, x, ..., x]` (of length `n`).

#### `find_by : [a : Eq] (a -> Bool) -> Array a -> Option I64`
Find the first index at which the element satisfies a condition.

#### `force_unique : Array a -> Array a`
Force the uniqueness of an array.
If the given array is shared, this function returns the cloned array.

#### `force_unique! : Array a -> Array a`
Force the uniqueness of an array.
If the given array is shared, this function panics.

#### `from_iter : Iterator a -> Array a`
Create an array from an iterator.

#### `from_map : I64 -> (I64 -> a) -> Array a`
Creates an array by a mapping function.
Example: `from_map(n, f) = [f(0), f(1), f(2), ..., f(n-1)]`.

#### `get_capacity : Array a -> I64`
Returns the capacity of an array.

#### `get_first : Array a -> Option a`
Get the first element of an array. Returns none if the array is empty.

#### `get_last : Array a -> Option a`
Get the last element of an array. Returns none if the array is empty.

#### `get_size : Array a -> I64`
Returns the length of an array.

#### `get_sub : I64 -> I64 -> Array a -> Array a`
`arr.get_sub(s, e)` returns an array `[ arr.@(i) | i ∈ [s, e) ]`, 
More precisely, let `N` denote the the size of the `arr`. 
Then `arr.get_sub(s, e)` returns `[ arr.@(s + i mod N) | i ∈ [0, n), n >= 0 is the minimum number such that s + n == e mod N ]`.

#### `is_empty : Array a -> Bool`
Returns if the array is empty or not.

#### `mod : I64 -> (a -> a) -> Array a -> Array a`
Modifies an array value by acting on an element at an index.
This function clones the given array if it is shared.
What is special about this function is that if you call `arr.mod(i, f)` when both of `arr` and `arr.@(i)` are unique, it is assured that `f` receives the element value which is unique. So `arr.mod(i, f)` is NOT equivalent to `let v = arr.@(i); arr.set(i, f(v))`.

#### `mod! : I64 -> (a -> a) -> Array a -> Array a`
Modifies an array value by acting on an element at an index.
This function never clones the given array. If the array is shared, this function panics. 
What is special about this function is that if you call `arr.mod(i, f)` when both of `arr` and `arr.@(i)` are unique, it is assured that `f` receives the element value which is unique. So `arr.mod(i, f)` is NOT equivalent to `let v = arr.@(i); arr.set(i, f(v))`.

#### `pop_back : Array a -> Array a`
Pop an element at the back of an array.
If the array is empty, this function does nothing.

#### `pop_back! : Array a -> Array a`
Pop an element at the back of an array.
If the array is empty, this function does nothing.
This function panics if elements must be cloned due to the given array being shared. 
Note that, when the given array is empty, this function will not panic even if it is shared.

#### `push_back : a -> Array a -> Array a`
Push an element to the back of an array.

#### `push_back! : a -> Array a -> Array a`
Push an element to the back of an array.
This function panics if elements must be cloned due to the given array being shared. 
Note that, when the capacity of `arr` is equal to its size, `arr.push_back!(e)` will not panic even if `arr` is shared because in this case cloning elements is inevitable whether or not `arr` is shared.

#### `reserve : I64 -> Array a -> Array a`
Reserves the memory region for an array.

#### `set : I64 -> a -> Array a -> Array a`
Updates a value of an element at an index of an array.
This function clones the given array if it is shared.

#### `set! : I64 -> a -> Array a -> Array a`
Updates a value of an element at an index of an array.
This function never clones the given array. If the given array is shared, this function panics.

#### `sort_by : ((a, a) -> Bool) -> Array a -> Array a`
Sort elements in an array by "less than" comparator.

#### `to_iter : Array a -> Iterator a`
Convert an array to an iterator.

#### `truncate : I64 -> Array a -> Array a`
Truncate an array, keeping the given number of first elements.
`truncante(len, arr)` does nothing if `len >= arr.get_size`.

#### `impl [a : Eq] Array a : Eq`

#### `impl Array : Functor`

#### `impl Array : Monad`

### Destructor

`Destructor a` is a boxed type which has two fields of type `a` and `a -> ()`, where the latter field is called destructor.
The destructor function will be called when a value of `Destructor a` is deallocated.
Note that the inner value of type `a` may be still alive after the destructor function is called.
This type is useful to manage resources allocated by C function.

```
type Destructor a = box struct { value : a, dtor : a -> () };
```

#### `borrow : (a -> b) -> Destructor a -> b`
Borrow the internal value.
`borrow(worker, dtor)` calls `worker` on the internal value captured by `dtor`, and returns the value returned by `worker`.
If you try to extract the value by `dtor.@_value` from `dtor : Destructor a` and this expression is the last use of `dtor`, 
then you get a value after the destructor function is called. 
On the other hand, in `borrow(worker, dtor)`, `worker` will be called before the destructor is called.

#### `make : a -> (a -> ()) -> Destructor a`
Make a destructor value.

### ErrMsg

A type (alias) for error message. 

```
type ErrMsg = String;
```

### IO

`IO a` is the type whose value represents an I/O action which returns a value of type `a`.

#### `_read_line_inner : Bool -> IOHandle -> IOFail ErrMsg String`
Read characters from an IOHandle.
If the first argument `upto_newline` is true, this function reads a file upto newline or EOF.

#### `_unsafe_perform : IO a -> a`
Perform the I/O action. This may violate purity of Fix.

#### `close_file : IOHandle -> IO ()`
Close a file.
Unlike C's `fclose`, closing an already closed `IOHandle` is safe and does nothing.

#### `eprint : String -> IO ()`
Print a string to stderr.

#### `eprintln : String -> IO ()`
Print a string followed by a newline to stderr.

#### `exit : I64 -> IO a`
Exit the program with an error code.

#### `exit_with_msg : I64 -> String -> IO a`
Exit the program with an error message and an error code.
The error message is written to the standard error output.

#### `input_line : IO String`
Read a line from stdin. If some error occurr, this function aborts.
If you want to handle errors, use `read_line(stdin)` instead.

#### `is_eof : IOHandle -> IO Bool`
Check if an `IOHandle` reached to the EOF.

#### `loop_lines : IOHandle -> s -> (s -> String -> LoopResult s s) -> IOFail s`
Loop on lines read from an `IOHandle`.
`loop_lines(handle, initial_state, worker)` calls `worker` on the pair of current state and a line string read from `handle`.
The function `worker` should return an updated state as `LoopResult` value, i.e., a value created by `continue` or `break`.
When the `handle` reaches to the EOF or `worker` returns a `break` value, `loop_lines` returns the last state value.
Note that the line string passed to `worker` may contain a newline code at the end. To remove it, use `String::strip_last_spaces`.

#### `loop_lines_io : IOHandle -> s -> (s -> String -> IOFail (LoopResult s s)) -> IOFail s`
Loop on lines read from an `IOHandle`.
Similar to `loop_lines`, but the worker function can perform an IO action.

#### `open_file : Path -> String -> IOFail IOHandle`
Open a file. The second argument is a mode string for `fopen` C function. 

#### `print : String -> IO ()`
Print a string to stdout.

#### `println : String -> IO ()`
Print a string followed by a newline to stdout.

#### `read_file_string : Path -> IOFail String`
Raad all characters from a file.

#### `read_file_bytes : Path -> IOFail (Array U8)`
Read all bytes from a file.

#### `read_line : IOHandle -> IOFail String`
Read characters from a IOHandle upto newline or EOF.
The returned string may include newline at its end.

#### `read_bytes : IOHandle -> IOFail (Array U8)`
Read all bytes from an IOHandle.

#### `read_n_bytes : IOHandle -> I64 -> IOFail (Array U8)`
Read at most n bytes from an IOHandle.

#### `read_string : IOHandle -> IOFail String`
Read all characters from a IOHandle.

#### `stderr : IOHandle`
The handle for standard error.

#### `stdin : IOHandle`
The handle for standard input.    

#### `stdout : IOHandle`
The handle for standard output.

#### `with_file : Path -> String -> (IOHandle -> IOFail a) -> IOFail a`
Perform a function with a file handle. The second argument is a mode string for `fopen` C function. 
The file handle will be closed automatically.

#### `write_bytes : IOHandle -> Array U8 -> IOFail ()`
Write a byte array into an IOHandle.

#### `write_file_bytes : Path -> Array U8 -> IOFail ()`
Write a byte array into a file.

#### `write_file_string : Path -> String -> IOFail ()`
Write a string into a file.

#### `write_string : IOHandle -> String -> IOFail ()`
Write a string into an IOHandle.

#### `impl IO : Functor`

#### `impl IO : Monad`

### IO::IOHandle
A handle type for read / write operations on files, stdin, stdout, stderr.
You can create `IOHandle` value by `IO::open_file`, and close it by `IO::close_file`. 
Also there are global `IO::IOHandle::stdin`, `IO::IOHandle::stdout`, `IO::IOHandle::stderr`.

#### `_file_ptr : IOHandle -> Ptr`.
Get pointer to C's `FILE` value from an `IOHandle`
DO NOT call `fclose` on the pointer returned by this function.
To close an `IOHandle`, use `IO::close_file`.

#### `_unsafe_close : IOHandle -> ()`
Close an `IOHandle`. 
This is an I/O action not wrapped by `IO`; use `IO::close_file` in the usual case.

### `from_file_ptr : Ptr -> IOHandle`
Create an `IOHandle` from a file pointer (i.e., pointer to C's `FILE`).
DO NOT create two `IOHandle`s from a single file pointer.

### IO::IOFail

The type for I/O actions which may fail.

```
type IOFail a = unbox struct { _data : IO (Result ErrMsg a) };
```

#### `from_result : Result ErrMsg a -> IOFail a`
Create an pure `IOFail` value from a `Result` value.

#### `lift : IO a -> IOFail a`
Lift an `IO` action to a successful `IOFail` action.

#### `throw : ErrMsg -> IOFail a`
Create an error `IOFail` action.

#### `to_result : IOFail a -> IO (Result ErrMsg a)`
Convert an `IOFail` to an `Result` value (wrapped by `IO`).

#### `try : (ErrMsg -> IO a) -> IOFail a -> IO a`
Convert an `IOFail` value to an `IO` value by an error handler (i.e., a `catch`) function.

#### `impl IOFail : Functor`

#### `impl IOFail : Monad`

### Iterator

Iterators (a.k.a. lazy lists) are generators of sequenced values.

#### `_flatten : Iterator (Iterator a) -> Iterator a`
Flatten an iterator of iterators.
You should use Monad::flatten instead of this function.
This function is used in the implementation of Monad::bind for Iterator.

#### `advance : Iterator a -> Option (a, Iterator a)`
Get next value and next iterator.

#### `append : Iterator a -> Iterator a -> Iterator a`
Append an iterator to a iterator.
Note: Since `iter1.append(iter2)` puts `iter2` after `iter1`, `append(lhs, rhs)` puts `lhs` after `rhs`.    

#### `bang : Iterator a -> Iterator a`
Evaluate all elements of iterator.

#### `count_up : I64 -> Iterator I64`
Create an iterator that counts up from a number.
Example: `count_up(n) = [n, n+1, n+2, ...]` (continues infinitely).

#### `empty : Iterator a`
Create an empty iterator.

#### `filter : (a -> Bool) -> Iterator a -> Iterator a`
Filter elements by a condition function.

#### `find_last : Iterator a -> Option a`
Takes the last element of an iterator.

#### `fold : b -> (b -> a -> b) -> Iterator a -> b`
Folds iterator from left to right.
Example: `fold(init, op, [a0, a1, a2, ...]) = ...op(op(op(init, a0), a1), a2)...`.

#### `fold_m : [m : Monad] b -> (b -> a -> m b) -> Iterator a -> m b`
Folds iterator from left to right by monadic action.

#### `from_array : Array a -> Iterator a`
Create iterator from an array.

#### `from_map : (I64 -> a) -> Iterator a`
Create iterator from mapping function.
Example: `from_map(f) = [f(0), f(1), f(2), ...]`.

#### `generate : s -> (s -> Option (a, s)) -> Iterator a`
Generate an iterator from a state transition function.
- if `f(s)` is none, `generate(s, f)` is empty.
- if `f(s)` is some value `(e, s1)`, then `generate(s, f)` starts by `e` followed by `generate(s2, f)`.

#### `get_first : Iterator a -> Option a`
Get the first element of an iterator. If the iterator is empty, this function returns `none`.

#### `get_size : Iterator a -> I64`
Count the number of elements of an iterator.

#### `get_tail : Iterator a -> Option (Iterator a)`
Remove the first element from an iterator. If the iterator is empty, this function returns `none`.

#### `intersperse : a -> Iterator a -> Iterator a`
Intersperse an elemnt between elements of an iterator.
Example: `Iterator::from_array([1,2,3]).intersperse(0) == Iterator::from_array([1,0,2,0,3])`

#### `is_empty : Iterator a -> Bool`
Check if the iterator is empty.

#### `loop_iter : b -> (b -> a -> LoopResult b b) -> Iterator a -> b`
Loop along an iterator. At each iteration step, you can choose to continue or to break.

#### `loop_iter_m : [m : Monad] b -> (b -> a -> m (LoopResult b b)) -> Iterator a -> m b`
Loop by monadic action along an iterator. At each iteration step, you can choose to continue or to break.

#### `push_front : a -> Iterator a -> Iterator a`
Push an element to an iterator.

#### `range : I64 -> I64 -> Iterator I64`
Create a range iterator, i.e. an iterator of the form `[a, a+1, a+2, ..., b-1]`.

#### `reverse : Iterator a -> Iterator a`
Reverse an iterator.

#### `subsequences : Iterator a -> Iterator (Iterator a)`
Generated all subsequences of an iterator.
For example, `[1,2,3].to_iter.subsequences` equals to `[[], [3], [2], [2, 3], [1], [1, 3], [1, 2], [1, 2, 3]].to_iter.map(to_iter)`.

#### `sum : [a : Additive] Iterator a -> a`
Calculate the sum of elements of an iterator.

#### `take : I64 -> Iterator a -> Iterator a`
Take at most n elements from an iterator.

#### `take_while : (a -> Bool) -> Iterator a -> Iterator a`
Take elements of an iterator while a condition is satisfied.

#### `to_array : Iterator a -> Array a`
Convert an iterator to an array.

#### `zip : Iterator b -> Iterator a -> Iterator (a, b)`
Zip two iterators.

#### `impl Iterator a : Add`
Adds two iterators by `Iterator::append`.

#### `impl [a : Eq] Iterator a : Eq`

#### `impl Iterator : Functor`

#### `impl Iterator : Monad`

### Lazy

The type of lazily generated values.
This is a type alias defined as `type Lazy a = () -> a;`
You can create a lazy value by `|_| (...an expression to generate the value...)`, and  
you can evaluate a lazy value `v` by `v()`.

### LoopResult

`LoopResult` represents the result of loop body function and used with `loop` function. For example of `LoopResult`, see the section for `loop` function.

```
type LoopResult s b = unbox union { continue : s, break : b };
```

#### `break_m : [m : Monad] r -> m (LoopResult s r)`
Make a break value wrapped in a monad. 
This is used with `loop_m` function.

#### `continue_m : [m : Monad] s -> m (LoopResult s r)`
Make a continue value wrapped in a monad.
This is used with `loop_m` function.

### Option

`Option a` contains a value of type `a`, or contains nothing.

```
type Option a = union { none: (), some: a };
```

#### `as_some_or : a -> Option a -> a`
Unwrap an option value if it is `some`, or returns given default value if it is `none`.

#### `map_or : b -> (a -> b) -> Option a -> b`
Returns the provided default value if the option is none, or applies a function to the contained value if the option is some.

#### `impl [a : Eq] Option a : Eq`

#### `impl Option : Functor`

#### `impl Option : Monad`

### Path

The type for file path.

NOTE: this type is prepared for future, and currently `Path` is only a wrapped `String`.

#### `parse : String -> Option Path`

Parse a string.

#### `impl Path : ToString`

### Ptr

`Ptr` is the type of pointers.

Literals:
- `nullptr`
    - The null pointer.

#### `add_offset : I64 -> Ptr -> Ptr`
Add an offset to a pointer.

#### `subtract_ptr : Ptr -> Ptr -> I64`
Subtract two pointers.
Note that `x.subtract_ptr(y)` calculates `x - y`, so `subtract_ptr(x, y)` calculates `y - x`.

#### `impl Ptr : Eq`
#### `impl Ptr : ToString`

### PunchedArray
The type of punched arrays. A punched array is an array from which a certain element has been removed.
If you create a punched array `parr` by punching an array `arr` at an index `idx`, only elements of `arr` whose indices are outside `idx` are released when `parr` is destructed.

```
type PunchedArray a = unbox struct { _data : Destructor (Array a), idx : I64 };
```

#### `plug_in! : a -> PunchedArray a -> Array a`
Plug in an element to a punched array to get back an array.
This function panics if (the internal data of) the given punched array is shared.

#### `punch! : I64 -> Array a -> (PunchedArray a, a)`
Creates a punched array.
Expression `punch(idx, arr)` evaluates to a pair `(parr, elm)`, where `elm` is the value that was stored at `idx` of `arr` and `parr` is the punched `arr` at `idx`.
This function panics if the given array is shared.

### Result

A type of result value for a computation that may fail.

```
type Result o e = unbox union { ok : o, err: e };
```

#### `unwrap : Result e o -> o`

Returns the containing value if the value is ok, or otherwise aborts.

#### `impl Result e : Monad`

### String

The type of strings.

#### `_unsafe_from_c_str : Array U8 -> String`
Create a string from C string (i.e., null-terminated byte array).
If the byte array doesn't include `\0`, this function causes undefined behavior.

#### `_unsafe_from_c_str_ptr : Ptr -> String`
Create a `String` from a pointer to null-terminated C string.
If `ptr` is not pointing to a valid null-terminated C string, this function cause undefined behavior.

#### `_get_c_str : String -> Ptr`
Get the null-terminated C string.
Note that in case the string is not used after call of this function, the returned pointer will be already released.

#### `borrow_c_str : (Ptr -> a) -> String -> a`
Call a function with a valid null-terminated C string.

#### `concat : String -> String -> String`
Concatenate two strings.
Note: Since `s1.concat(s2)` puts `s2` after `s1`, `concat(lhs, rhs)` puts `lhs` after `rhs`.

#### `concat_iter : Iterator String -> String`
Concatenate an iterator of strings.

#### `empty : I64 -> String`
Create an empty string, which is reserved for a length.

#### `find : String -> I64 -> String -> Option I64`
`str.find(token, start_idx)` finds the index where `token` firstly appears in `str`, starting from `start_idx`.
Note that this function basically returns a number less than or equal to `start_idx`, but there is an exception:
`str.find("", start_idx)` with `start_idx >= str.get_size` returns `str.get_size`, not `start_idx`.

#### `get_bytes : String -> Array U8`
Get the byte array of a string, containing null-terminator.

#### `get_first_byte : String -> Option Byte`
Get the first byte of a string. Returns none if the string is empty.

#### `get_last_byte : String -> Option Byte`
Get the last byte of a string. Returns none if the string is empty.

#### `get_size : String -> I64`
Returns the length of the string.

#### `get_sub : I64 -> I64 -> String -> String`
`String` version of `Array::get_sub`.

#### `is_empty : String -> Bool`
Returns if the string is empty or not.

#### `join : String -> Iterator String -> String`
Join strings by a separator.
Example: `Iterator::from_array(["a", "b", "c"]).join(", ") == "a, b, c"`

#### `pop_back_byte : String -> String`
Removes the last byte.
If the string is empty, this function does nothing.

#### `split : String -> String -> Iterator String`
`str.split(sep)` splits `str` by `sep` into an iterator.
- If `sep` is empty, this function returns an infinite sequence of ""s.
- If `sep` is non-empty and `str` is empty, this function returns an iterator with a single element "".

#### `strip_first_bytes : (U8 -> Bool) -> String -> String`
Removes the first byte of a string while it satisifies the specified condition.

#### `strip_first_spaces : String -> String`
Removing leading whitespace characters.

#### `strip_last_bytes : (U8 -> Bool) -> String -> String`
Removes the last byte of a string while it satisifies the specified condition.

#### `strip_last_newlines : String -> String`
Removes newlines and carriage returns at the end of the string.

#### `strip_last_spaces : String -> String`
Removing trailing whitespace characters.

#### `strip_spaces : String -> String`
Strip leading and trailing whitespace characters.

#### `impl String : Add`
Add two strings by `String.concat`.

#### `impl String : Eq`

#### `impl String : ToString`
Defined as an identity function.

### Tuple{N}

#### `impl [a : Eq, b : Eq] (a, b) : Eq`

#### `impl [a : ToString, b : ToString] (a, b) : ToString`

### `()`

The unit type which has a unique value also written as `()`.

Literals:
* `() : ()` represents the unique value of type `()`.

#### `impl () : Eq`

## Functions

### `abort : Lazy a`

Evaluating this value stops the execution of the program.

### `compose : (a -> b) -> (b -> c) -> a -> c`

Compose two functions. Composition operators `<<` and `>>` is translated to use of `compose`. 

### `fix : ((a -> b) -> a -> b) -> a -> b`

`fix` enables you to make a recursive function locally. The idiom is: `fix $ |loop, var| -> (expression calls loop)`.

```
module Main;

main : IO ();
main = (
    let fact = fix $ |loop, n| if n == 0 { 1 } else { n * loop (n-1) };
    println $ fact(5).to_string // evaluates to 5 * 4 * 3 * 2 * 1 = 120
);
```

### `loop : s -> (s -> LoopResult s r) -> r`

`loop` enables you to make a loop. `LoopResult` is a union type defined as follows: 

```
type LoopResult s r = unbox union { s: continue, r: break };
```

`loop` takes two arguments: the initial state of the loop `s0` and the loop body function `body`. It first calls `body` on `s0`. If `body` returns `break r`, then the loop ends and returns `r` as the result. If `body` returns `continue s`, then the loop calls again `body` on `s`.

```
module Main;
    
main : IO ();
main = (
    let sum = (
        loop((0, 0), |(i, sum)|
            if i == 100 {
                break $ sum 
            } else {
                continue $ (i+1, sum+i)
            }
        )
    );
    println $ sum.to_string
); // evaluates to 0 + 1 + ... + 99 
```

### `loop_m : [m : Monad] s -> (s -> m (LoopResult s r)) -> m r`

Monadic loop function. This is similar to `loop` but can be used to perform monadic action at each loop.

It is convenient to use `continue_m` and `break_m` to create monadic loop body function.

The following program prints "Hello World! (i)" for i = 0, 1, 2.

```
module Main;

main : IO ();
main = (
    loop_m(0, |i| (
        if i == 3 { break_m $ () };
        eval *println("Hello World! (" + i.to_string + ")");
        continue_m $ i + 1
    ))
);
```

### `unsafe_is_unique : a -> (Bool, a)`

This function checks if a value is uniquely refernced by a name, and returns the result paired with the given value itself. If `a` is unboxed, the 0th component of the returned value is always `true`.

NOTE: Using the return value of this function to branch and change the return value of your function may break the referential transparency of the function. If you want to panic when a value is shared, consider using `Debug::assert_unique!` instead.

Example: 

```
module Main;

import Debug;

main : IO ();
main = (
    // For unboxed value, it returns true even if the value is used later.
    let int_val = 42;
    let (unique, _) = int_val.unsafe_is_unique;
    let use = int_val + 1;
    eval assert_eq(|_|"fail: int_val is shared", unique, true);

    // For boxed value, it returns true if the value isn't used later.
    let arr = Array::fill(10, 10);
    let (unique, arr) = arr.unsafe_is_unique;
    let use = arr.@(0); // This `arr` is not the one passed to `is_unique`, but the one returned by `is_unique`.
    eval assert_eq(|_|"fail: arr is shared", unique, true);

    // Fox boxed value, it returns false if the value will be used later.
    let arr = Array::fill(10, 10);
    let (unique, _) = arr.unsafe_is_unique;
    let use = arr.@(0);
    eval assert_eq(|_|"fail: arr is unique", unique, false);

    pure()
);
```

## Traits

### Additive

`Additive` is a trait alias defined as follows:

```
trait Additive = Add + Zero;
```

### FromBytes

`a : FromBytes` states that a byte array can be parsed to create a value of type `a`.

NOTE: For primitive types of Fix, `FromBytes` is implemented as bitwise casting.

#### `from_bytes : [a : FromBytes] Array U8 -> Result ErrMsg a`

### FromString

`a : FromString` states that strings can be parsed to create a value of type `a`.

NOTE: For primitive types of Fix, `FromString` is implemented using `strto**` functions of C, but they do not accept whitespace prefix or suffix.

#### `from_string : [a : FromString] String -> Result ErrMsg a`

### Functor (* -> *)

#### `map : [f : Functor] (a -> b) -> f a -> f b`

### LessThan

#### `less_than : [a : LessThan] a -> a -> a`

#### `max : [a : LessThan] a -> a -> a`

#### `min : [a : LessThan] a -> a -> a`

### LessThanOrEq

#### `less_than_or_eq : [a : LessThanOrEq] a -> a -> a`

### Monad (* -> *)

#### (required) `bind : [m : Monad] (a -> m b) -> m a -> m b`

#### (required) `pure : [m : Monad] a -> m a`

#### `flatten : [m : Monad] m (m a) -> a`

This is equivalent to `Monad::bind(|x|x)`.

### ToBytes

#### `to_bytes : [a : ToBytes] a -> Array U8`

### ToString

#### `to_string : [a: ToString] a -> String`

# Module `Character`
This module provides wrapper functions of C functions defined in ctypes.h.

## `is_alnum : U8 -> Bool`

## `is_alpha : U8 -> Bool`

## `is_blank : U8 -> Bool`

## `is_cntrl : U8 -> Bool`

## `is_digit : U8 -> Bool`

## `is_graph : U8 -> Bool`

## `is_lower : U8 -> Bool`

## `is_print : U8 -> Bool`

## `is_punct : U8 -> Bool`

## `is_space : U8 -> Bool`

## `is_upper : U8 -> Bool`

## `is_xdigit : U8 -> Bool`

## `to_lower : U8 -> U8`

## `to_upper : U8 -> U8`

# Module `Debug`
Collection of functions for debugging.
This module contains fucntions violating purity, e.g., printing string to stdio without IO monad.

## `_debug_print_to_stream : IOHandle -> String -> ()`
Prints a string to the specified stream and flushes the stream.

## `assert : Lazy String -> Bool -> ()`
Asserts that a condition (boolean value) is true.
If the assertion failed, prints a message to the stderr and aborts.

## `assert_eq : [a: Eq] Lazy String -> a -> a -> ()`
Asserts that two values are equal.
If the assertion failed, prints a message to the stderr and aborts.

## `assert_unique! : Lazy String -> a -> a`
Asserts that the given value is unique, and returns the given value.
If the assertion failed, prints a message to the stderr and aborts.
The main use of this function is to check whether a boxed value given as an argument is unique.

## `consumed_time_while : (a -> b) -> a -> (b, F64)`
Get clocks (cpu time) elapsed while executing a function.

## `consumed_time_while_io : IO a -> IO (a, F64)`
Get clocks (cpu time) elapsed while executing an I/O action.

## `consumed_time_while_lazy : Lazy a -> (a, F64)`
Get clocks (cpu time) elapsed while evaluating a lazy value.

## `debug_eprint : String -> ()`
Prints a string to stderr and flushes.

## `debug_eprintln : String -> ()`
Prints a string followed by a newline to stderr and flushes.

## `debug_print : String -> ()`
Prints a string to stdout and flushes.

## `debug_println : String -> ()`
Prints a string followed by a newline to stdout and flushes.

# Module `Hash`

## `trait a : Hash`

## `trait HashKey = Hash + Eq`
Trait required for keys of HashSet and HashMap.

### (required) `hash : [a : Hash] a -> U64`

## `impl [a : Hash] Array a : Hash`
This is implemented by djb2 algorithm, although I don't know whether it is effective for not only strings (i.e., `U8` array) but also `U64` (result type of hash function) array!

## `impl String : Hash`

## `impl U8 : Hash`

## `impl I64 : Hash`

## `impl U64 : Hash`

## `impl [a : Hash, b : Hash] (a, b) : Hash`

# Module `HashMap`

## `type HashMap k v`
`HashMap` is a structure that stores key-value pairs into hash tables.

## `_find_place : [k : HashKey] k -> HashMap k v -> (I64, Option I64)`
Find the place where an element with a key is stored.
Returns pair of (index in hash table, index in bucket).

## `_get_pot_geq : I64 -> I64`
Get a POT (power-of-two) value which is less than or equal to the given value.
This is used for calculating capacity value.

## `contains_key : [k : HashKey] k -> HashMap k v -> Bool`
Check whether a hashmap contains a key.

## `empty : I64 -> HashMap k v`
Create an empty HashMap which is reserved so that it will not rehash until size exceeds the spacified value.

## `erase : [k : HashKey] k -> HashMap k v -> HashMap k v`
Erase an element from a HashMap.

## `find : [k : HashKey] k -> HashMap k v -> Option v`
Find an element from a HashMap.

## `find_or : [k : HashKey] k -> v -> HashMap k v -> Option v`
Find an element from a HashMap. If the map doesn't contain the key, it returns the given default value.

## `get_capacity : HashMap k v -> I64`
Get capacity of a HashMap. 

## `get_size : HashMap k v -> I64`
Get size (number of elements) in a HashMap.

## `insert : [k : HashKey] k -> v -> HashMap k v -> HashMap k v`
Insert an element into a HashMap.

## `reserve : [k : HashKey] I64 -> HashMap k v -> HashMap k v`
Reserve a HashMap so that it will not rehash until size exceeds the spacified value.

## `to_iter : HashMap k v -> Iterator (k, v)`
Convert a HashMap into an iterator.

# Module `HashSet`

## `type HashSet k`
`HashSet` is a structure that stores elements into hash tables.

## `contains : [k : HashKey] k -> HashSet k -> Bool`
Check whether a hashset contains an element.

## `empty : I64 -> HashSet k`
Create an empty HashSet which is reserved so that it will not rehash until size exceeds the spacified value.

## `erase : [k : HashKey] k -> HashSet k -> HashSet k`
Erase an element from a HashSet. 

## `from_iter : [k : HashKey] Iterator k -> HashSet k`
Construct a HashSet from an iterator of elements.

## `get_capacity : HashSet k -> I64`
Get capacity of a HashSet.

## `get_size : HashSet k -> I64`
Get size (number of elements) of a HashSet.

## `insert : [k : HashKey] k -> HashSet k -> HashSet k`
Insert an element into a HashSet.

## `intersect : [k : HashKey] HashSet k -> HashSet k -> HashSet k`
Calculate intersection of two Hashsets.

## `merge : [k : HashKey] HashSet k -> HashSet k -> HashSet k`
Calculate union of two HashSets.

## `reserve : [k : HashKey] I64 -> HashSet k -> HashSet k`
Reserve a HashSet so that it will not rehash until size exceeds the spacified value.

## `to_iter : HashSet k -> Iterator k`
Convert a HashSet into an iterator.

# Module `Math`
// A math library.
// Importing this module automatically links libm.so to the program.

## `_gcd_nonneg : I64 -> I64 -> I64`
Calculate greatest common divisors of two non-negative integers. 

## `acos : F64 -> F64`
Calculate arc cosine of the argument.
This is wrapper of C's acos.

## `asin : F64 -> F64`
Calculate arc sine of the argument.
This is wrapper of C's asin.

## `atan : F64 -> F64`
Calculate arc tangent of the argument.
This is wrapper of C's atan.

## `atan2 : F64 -> F64 -> F64`
Calculate arc tangent of y/x for argument x, y.
This is wrapper of C's atan2.

## `binomial_coefficients : I64 -> Array (Array I64)`
Calculate table (2-dimensional array) of binomial coefficients.
`binomial_coefficients(m)` evaluates to an array of arrays `table` where `table.@(n).@(r)` is the binomial coefficient "binom(n, r)" for 0 <= n <= m and 0 <= r <= n.
Here `m` has to be less than or equal to 66 to avoid overflow.

## `ceil : F64 -> F64`
Calculate the smallest integral value not less than the argument.
This is wrapper of C's ceil.

## `cos : F64 -> F64`
Calculate the cosine of the argument.
This is wrapper of C's cos.

## `cosh : F64 -> F64`
Calculate the hyperbolic cosine of the argument.
This is wrapper of C's cosh.

## `exp : F64 -> F64`
Calculate the natural exponential of the argument.
This is wrapper of C's exp.

## `floor : F64 -> F64`
Calculate the largest integral value not greater than the argument.
This is wrapper of C's floor.

## `fmod : F64 -> F64 -> F64`
Calculate the floating point remainder of division. 
`x.fmod(y)` evaluates to the remainder of dividing x by y.
This is wrapper of C's fmod.

## `frexp : F64 -> (F64, I32)`
Split a floating point number to normalized fraction and an exponent.
This is wrapper of C's frexp. 

## `gcd : I64 -> I64 -> I64`
Calculate greatest common divisor of two integers. `gcd(0, 0)` returns `0`.
NOTE: currently, this function does not support `I64::minimum`.

## `ldexp : I32 -> F64 -> F64`
Multiply a floating point number by power of two.
This is wrapper of C's ldexp. 

## `log : F64 -> F64`
Calculate natural logarithm.
This is wrapper of C's log. 

## `log10 : F64 -> F64`
Calculate base-10 logarithm.
This is wrapper of C's log10. 

## `modf : F64 -> (F64, F64)`
Convert a floating pointer number into the pair of fractional part and integral part.
This is wrapper of C's modf.

## `pi32 : F32`
Pi as `F32`

## `pi64 : F64`
Pi as `F64`

## `pow : F64 -> F64 -> F64`
Power function.
`x.pow(y)` evaluates to x^y.
This is wrapper of C's pow.

## `sin : F64 -> F64`
Calculate the sine of the argument.
This is wrapper of C's sin.

## `sinh : F64 -> F64`
Calculate the hyperbolic sine of the argument.
This is wrapper of C's sinh.

## `sqrt : F64 -> F64`
Calculate square root of the argument.
This is wrapper of C's sqrt.

## `tan : F64 -> F64`
Calculate the tangent of the argument.
This is wrapper of C's tan.

## `tanh : F64 -> F64`
Calculate the hyperbolic tangent of the argument.
This is wrapper of C's tanh.

# module `Subprocess`

## `type ExitStatus`
This type represents the exit status of a subprocess.
This type is the union of following variants:
* `exit : U8` - Means that the subprocess successfully exited (i.e., the main function returned or `exit()` was called) and stores the exit status code.
* `signaled : U8` - Means that the subprocess was terminated by a signal and stores the signal number which caused the termination.
* `wait_failed : ()` - Means that the `run*` function failed to wait the subprocess to exit.

## `run_string : String -> Array String -> String -> IOFail ((String, String), ExitStatus)`
`run_string(com, args, input)` executes a command specified by `com` with arguments `args`, and writes `input` to the standard input of the running command.
The result is the pair of standard output and standard error, and an `ExitStatus` value.

## `run_with_stream : String -> Array String -> ((IOHandle, IOHandle, IOHandle) -> IOFail a) -> IOFail (a, ExitStatus)`
`run_with_stream(com, args, worker)` executes a command specified by `com` with arguments `args`. 
The function `worker` receives three `IOHandle`s which are piped to the stdin, stdout and stderr of the running command.
The result is the value returned by `worker` paired with an `ExitStatus` value.
* `com : String` - The path to the program to run.
* `args: Array String` - The arguments to be passed to `com`.
* `worker : (IOHandle, IOHandle, IOHandle) -> IOFail a` - Receives three `IOHandle`s which are piped to stdin, stdout and stderr of the running command.

# module `Time`

## `type Time`

The type that represents time by the number of seconds and micro seconds elapsed since the unix epoch.
This struct has two fields, `sec: I64` and `nanosec: U32`.
```
type Time = unbox struct { sec : I64, nanosec : U32 };
```

## `type DateTime`
The type to represent date and time.
```
type DateTime = unbox struct {
    nanosec : U32, // [0-99999]
    sec : U8, // [0-61]
    min : U8, // [0-59]
    hour : U8, // [0-23]
    day_in_month : U8, // [1-31]
    month : U8, // [1-12]
    day_in_week : U8, // [0: Sun, ..., 6: Sat]
    day_in_year : U32, // [1-366] TODO: we will change this to U16 in a future.
    year : I32,
    is_dst : Option Bool // Whether or not this datetime is under daylight saving time. `none` implies unknown/unspecified.
};
```

## `_datetime_to_time_inner : Bool -> DateTime -> Result ErrMsg Time`
Convert datetime to time.
`_datetime_to_time_inner(false)` treats the argument as UTC datetime, and `_datetime_to_time_inner(true)` treats the argument as local datetime.
Note that "local time" depends on timezone, so this function is violating purity.

## `_time_to_datetime_inner : Bool -> Time -> Result ErrMsg DateTime`
Convert time to datetime. 
`_time_to_datetime_inner(false)` returns utc datetime, and `_time_to_datetime_inner(true)` returns local datetime.
Note that "local time" depends on timezone, so this function is violating purity.

## `from_local : DateTime -> IOResult ErrMsg Time`
Convert local datetime to time.
This function depends on timezone, so it returns `IOResult` value.

## `from_utc : DateTime -> Result ErrMsg Time`
Convert UTC datetime to time.

## `get_now : IO Time`
Get current time.

## `to_F64 : Time -> F64`
Convert time to 64-bit floating value.

## `to_local : Time -> IOResult ErrMsg DateTime`
Convert time to local time.
This function depends on timezone, so it returns `IOResult` value.

## `to_utc : Time -> Result ErrMsg DateTime`
Convert time to UTC datetime.