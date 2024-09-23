# `module Std`

- [`module Std`](#module-std)
- [Types and aliases](#types-and-aliases)
  - [`namespace Std`](#namespace-std)
    - [`type Array a = box { primitive }`](#type-array-a--box--primitive-)
    - [`type Bool = unbox { primitive }`](#type-bool--unbox--primitive-)
    - [`type Boxed a = box struct { ...fields... }`](#type-boxed-a--box-struct--fields-)
      - [field `value : a`](#field-value--a)
    - [`type ErrMsg = String`](#type-errmsg--string)
    - [`type F32 = unbox { primitive }`](#type-f32--unbox--primitive-)
    - [`type F64 = unbox { primitive }`](#type-f64--unbox--primitive-)
    - [`type I16 = unbox { primitive }`](#type-i16--unbox--primitive-)
    - [`type I32 = unbox { primitive }`](#type-i32--unbox--primitive-)
    - [`type I64 = unbox { primitive }`](#type-i64--unbox--primitive-)
    - [`type I8 = unbox { primitive }`](#type-i8--unbox--primitive-)
    - [`type IO a = unbox struct { ...fields... }`](#type-io-a--unbox-struct--fields-)
      - [field `_data : () -> a`](#field-_data-----a)
    - [`type Iterator a = unbox struct { ...fields... }`](#type-iterator-a--unbox-struct--fields-)
      - [field `next : () -> Option (a, Iterator a)`](#field-next-----option-a-iterator-a)
    - [`type Lazy = () -> a`](#type-lazy-----a)
    - [`type LoopResult s b = unbox union { ...variants... }`](#type-loopresult-s-b--unbox-union--variants-)
      - [variant `continue : s`](#variant-continue--s)
      - [variant `break : b`](#variant-break--b)
    - [`type Option a = unbox union { ...variants... }`](#type-option-a--unbox-union--variants-)
      - [variant `none : ()`](#variant-none--)
      - [variant `some : a`](#variant-some--a)
    - [`type Path = unbox struct { ...fields... }`](#type-path--unbox-struct--fields-)
      - [field `_data : String`](#field-_data--string)
    - [`type Ptr = unbox { primitive }`](#type-ptr--unbox--primitive-)
    - [`type PunchedArray a = unbox struct { ...fields... }`](#type-punchedarray-a--unbox-struct--fields-)
      - [field `_data : Destructor (Array a)`](#field-_data--destructor-array-a)
      - [field `idx : I64`](#field-idx--i64)
    - [`type Result e o = unbox union { ...variants... }`](#type-result-e-o--unbox-union--variants-)
      - [variant `ok : o`](#variant-ok--o)
      - [variant `err : e`](#variant-err--e)
    - [`type String = unbox struct { ...fields... }`](#type-string--unbox-struct--fields-)
      - [field `_data : Array U8`](#field-_data--array-u8)
    - [`type U16 = unbox { primitive }`](#type-u16--unbox--primitive-)
    - [`type U32 = unbox { primitive }`](#type-u32--unbox--primitive-)
    - [`type U64 = unbox { primitive }`](#type-u64--unbox--primitive-)
    - [`type U8 = unbox { primitive }`](#type-u8--unbox--primitive-)
  - [`namespace Std::FFI`](#namespace-stdffi)
    - [`type CChar = Std::I8`](#type-cchar--stdi8)
    - [`type CDouble = Std::F64`](#type-cdouble--stdf64)
    - [`type CFloat = Std::F32`](#type-cfloat--stdf32)
    - [`type CInt = Std::I32`](#type-cint--stdi32)
    - [`type CLong = Std::I64`](#type-clong--stdi64)
    - [`type CLongLong = Std::I64`](#type-clonglong--stdi64)
    - [`type CShort = Std::I16`](#type-cshort--stdi16)
    - [`type CSizeT = Std::U64`](#type-csizet--stdu64)
    - [`type CUnsignedChar = Std::U8`](#type-cunsignedchar--stdu8)
    - [`type CUnsignedInt = Std::U32`](#type-cunsignedint--stdu32)
    - [`type CUnsignedLong = Std::U64`](#type-cunsignedlong--stdu64)
    - [`type CUnsignedLongLong = Std::U64`](#type-cunsignedlonglong--stdu64)
    - [`type CUnsignedShort = Std::U16`](#type-cunsignedshort--stdu16)
    - [`type Destructor a = box struct { ...fields... }`](#type-destructor-a--box-struct--fields-)
      - [field `_value : a`](#field-_value--a)
      - [field `dtor : a -> ()`](#field-dtor--a---)
  - [`namespace Std::IO`](#namespace-stdio)
    - [`type IOFail a = unbox struct { ...fields... }`](#type-iofail-a--unbox-struct--fields-)
      - [field `_data : IO (Result ErrMsg a)`](#field-_data--io-result-errmsg-a)
    - [`type IOHandle = unbox struct { ...fields... }`](#type-iohandle--unbox-struct--fields-)
      - [field `_data : Destructor Ptr`](#field-_data--destructor-ptr)
- [Traits and aliases](#traits-and-aliases)
  - [`namespace Std`](#namespace-std-1)
    - [`trait a : Add`](#trait-a--add)
      - [method `add : a -> a -> a`](#method-add--a---a---a)
    - [`trait a : Div`](#trait-a--div)
      - [method `div : a -> a -> a`](#method-div--a---a---a)
    - [`trait a : Eq`](#trait-a--eq)
      - [method `eq : a -> a -> Bool`](#method-eq--a---a---bool)
    - [`trait a : FromBytes`](#trait-a--frombytes)
      - [method `from_bytes : Array U8 -> Result ErrMsg a`](#method-from_bytes--array-u8---result-errmsg-a)
    - [`trait a : FromString`](#trait-a--fromstring)
      - [method `from_string : String -> Result ErrMsg a`](#method-from_string--string---result-errmsg-a)
    - [`trait [f : *->*] f : Functor`](#trait-f----f--functor)
      - [method `map : (a -> b) -> f a -> f b`](#method-map--a---b---f-a---f-b)
    - [`trait a : LessThan`](#trait-a--lessthan)
      - [method `less_than : a -> a -> Bool`](#method-less_than--a---a---bool)
    - [`trait a : LessThanOrEq`](#trait-a--lessthanoreq)
      - [method `less_than_or_eq : a -> a -> Bool`](#method-less_than_or_eq--a---a---bool)
    - [`trait [m : *->*] m : Monad`](#trait-m----m--monad)
      - [method `bind : (a -> m b) -> m a -> m b`](#method-bind--a---m-b---m-a---m-b)
      - [method `pure : a -> m a`](#method-pure--a---m-a)
    - [`trait a : Mul`](#trait-a--mul)
      - [method `mul : a -> a -> a`](#method-mul--a---a---a)
    - [`trait a : Neg`](#trait-a--neg)
      - [method `neg : a -> a`](#method-neg--a---a)
    - [`trait a : Not`](#trait-a--not)
      - [method `not : a -> a`](#method-not--a---a)
    - [`trait a : Rem`](#trait-a--rem)
      - [method `rem : a -> a -> a`](#method-rem--a---a---a)
    - [`trait a : Sub`](#trait-a--sub)
      - [method `sub : a -> a -> a`](#method-sub--a---a---a)
    - [`trait a : ToBytes`](#trait-a--tobytes)
      - [method `to_bytes : a -> Array U8`](#method-to_bytes--a---array-u8)
    - [`trait a : ToString`](#trait-a--tostring)
      - [method `to_string : a -> String`](#method-to_string--a---string)
    - [`trait a : Zero`](#trait-a--zero)
      - [method `zero : a`](#method-zero--a)
- [Trait implementations](#trait-implementations)
    - [`impl () : Eq`](#impl---eq)
    - [`impl () : ToString`](#impl---tostring)
    - [`impl Array : Functor`](#impl-array--functor)
    - [`impl Array : Monad`](#impl-array--monad)
    - [`impl [a : Eq] Array a : Eq`](#impl-a--eq-array-a--eq)
    - [`impl [a : Eq, a : LessThan] Array a : LessThan`](#impl-a--eq-a--lessthan-array-a--lessthan)
    - [`impl [a : Eq, a : LessThanOrEq] Array a : LessThanOrEq`](#impl-a--eq-a--lessthanoreq-array-a--lessthanoreq)
    - [`impl [a : ToString] Array a : ToString`](#impl-a--tostring-array-a--tostring)
    - [`impl Bool : ToString`](#impl-bool--tostring)
    - [`impl F32 : FromBytes`](#impl-f32--frombytes)
    - [`impl F32 : FromString`](#impl-f32--fromstring)
    - [`impl F32 : ToBytes`](#impl-f32--tobytes)
    - [`impl F32 : ToString`](#impl-f32--tostring)
    - [`impl F32 : Zero`](#impl-f32--zero)
    - [`impl F64 : FromBytes`](#impl-f64--frombytes)
    - [`impl F64 : FromString`](#impl-f64--fromstring)
    - [`impl F64 : ToBytes`](#impl-f64--tobytes)
    - [`impl F64 : ToString`](#impl-f64--tostring)
    - [`impl F64 : Zero`](#impl-f64--zero)
    - [`impl I16 : FromBytes`](#impl-i16--frombytes)
    - [`impl I16 : FromString`](#impl-i16--fromstring)
    - [`impl I16 : ToBytes`](#impl-i16--tobytes)
    - [`impl I16 : ToString`](#impl-i16--tostring)
    - [`impl I16 : Zero`](#impl-i16--zero)
    - [`impl I32 : FromBytes`](#impl-i32--frombytes)
    - [`impl I32 : FromString`](#impl-i32--fromstring)
    - [`impl I32 : ToBytes`](#impl-i32--tobytes)
    - [`impl I32 : ToString`](#impl-i32--tostring)
    - [`impl I32 : Zero`](#impl-i32--zero)
    - [`impl I64 : FromBytes`](#impl-i64--frombytes)
    - [`impl I64 : FromString`](#impl-i64--fromstring)
    - [`impl I64 : ToBytes`](#impl-i64--tobytes)
    - [`impl I64 : ToString`](#impl-i64--tostring)
    - [`impl I64 : Zero`](#impl-i64--zero)
    - [`impl I8 : FromBytes`](#impl-i8--frombytes)
    - [`impl I8 : FromString`](#impl-i8--fromstring)
    - [`impl I8 : ToBytes`](#impl-i8--tobytes)
    - [`impl I8 : ToString`](#impl-i8--tostring)
    - [`impl I8 : Zero`](#impl-i8--zero)
    - [`impl IO : Functor`](#impl-io--functor)
    - [`impl IO : Monad`](#impl-io--monad)
    - [`impl IOFail : Functor`](#impl-iofail--functor)
    - [`impl IOFail : Monad`](#impl-iofail--monad)
    - [`impl Iterator : Functor`](#impl-iterator--functor)
    - [`impl Iterator : Monad`](#impl-iterator--monad)
    - [`impl Iterator a : Add`](#impl-iterator-a--add)
    - [`impl [a : Eq] Iterator a : Eq`](#impl-a--eq-iterator-a--eq)
    - [`impl Option : Functor`](#impl-option--functor)
    - [`impl Option : Monad`](#impl-option--monad)
    - [`impl [a : Eq] Option a : Eq`](#impl-a--eq-option-a--eq)
    - [`impl [a : ToString] Option a : ToString`](#impl-a--tostring-option-a--tostring)
    - [`impl Path : ToString`](#impl-path--tostring)
    - [`impl Ptr : ToString`](#impl-ptr--tostring)
    - [`impl Result e : Functor`](#impl-result-e--functor)
    - [`impl Result e : Monad`](#impl-result-e--monad)
    - [`impl [e : Eq, a : Eq] Result e a : Eq`](#impl-e--eq-a--eq-result-e-a--eq)
    - [`impl [e : ToString, a : ToString] Result e a : ToString`](#impl-e--tostring-a--tostring-result-e-a--tostring)
    - [`impl Std::Bool : Std::Eq`](#impl-stdbool--stdeq)
    - [`impl Std::Bool : Std::Not`](#impl-stdbool--stdnot)
    - [`impl Std::F32 : Std::Add`](#impl-stdf32--stdadd)
    - [`impl Std::F32 : Std::Div`](#impl-stdf32--stddiv)
    - [`impl Std::F32 : Std::Eq`](#impl-stdf32--stdeq)
    - [`impl Std::F32 : Std::LessThan`](#impl-stdf32--stdlessthan)
    - [`impl Std::F32 : Std::LessThanOrEq`](#impl-stdf32--stdlessthanoreq)
    - [`impl Std::F32 : Std::Mul`](#impl-stdf32--stdmul)
    - [`impl Std::F32 : Std::Neg`](#impl-stdf32--stdneg)
    - [`impl Std::F32 : Std::Sub`](#impl-stdf32--stdsub)
    - [`impl Std::F64 : Std::Add`](#impl-stdf64--stdadd)
    - [`impl Std::F64 : Std::Div`](#impl-stdf64--stddiv)
    - [`impl Std::F64 : Std::Eq`](#impl-stdf64--stdeq)
    - [`impl Std::F64 : Std::LessThan`](#impl-stdf64--stdlessthan)
    - [`impl Std::F64 : Std::LessThanOrEq`](#impl-stdf64--stdlessthanoreq)
    - [`impl Std::F64 : Std::Mul`](#impl-stdf64--stdmul)
    - [`impl Std::F64 : Std::Neg`](#impl-stdf64--stdneg)
    - [`impl Std::F64 : Std::Sub`](#impl-stdf64--stdsub)
    - [`impl Std::I16 : Std::Add`](#impl-stdi16--stdadd)
    - [`impl Std::I16 : Std::Div`](#impl-stdi16--stddiv)
    - [`impl Std::I16 : Std::Eq`](#impl-stdi16--stdeq)
    - [`impl Std::I16 : Std::LessThan`](#impl-stdi16--stdlessthan)
    - [`impl Std::I16 : Std::LessThanOrEq`](#impl-stdi16--stdlessthanoreq)
    - [`impl Std::I16 : Std::Mul`](#impl-stdi16--stdmul)
    - [`impl Std::I16 : Std::Neg`](#impl-stdi16--stdneg)
    - [`impl Std::I16 : Std::Rem`](#impl-stdi16--stdrem)
    - [`impl Std::I16 : Std::Sub`](#impl-stdi16--stdsub)
    - [`impl Std::I32 : Std::Add`](#impl-stdi32--stdadd)
    - [`impl Std::I32 : Std::Div`](#impl-stdi32--stddiv)
    - [`impl Std::I32 : Std::Eq`](#impl-stdi32--stdeq)
    - [`impl Std::I32 : Std::LessThan`](#impl-stdi32--stdlessthan)
    - [`impl Std::I32 : Std::LessThanOrEq`](#impl-stdi32--stdlessthanoreq)
    - [`impl Std::I32 : Std::Mul`](#impl-stdi32--stdmul)
    - [`impl Std::I32 : Std::Neg`](#impl-stdi32--stdneg)
    - [`impl Std::I32 : Std::Rem`](#impl-stdi32--stdrem)
    - [`impl Std::I32 : Std::Sub`](#impl-stdi32--stdsub)
    - [`impl Std::I64 : Std::Add`](#impl-stdi64--stdadd)
    - [`impl Std::I64 : Std::Div`](#impl-stdi64--stddiv)
    - [`impl Std::I64 : Std::Eq`](#impl-stdi64--stdeq)
    - [`impl Std::I64 : Std::LessThan`](#impl-stdi64--stdlessthan)
    - [`impl Std::I64 : Std::LessThanOrEq`](#impl-stdi64--stdlessthanoreq)
    - [`impl Std::I64 : Std::Mul`](#impl-stdi64--stdmul)
    - [`impl Std::I64 : Std::Neg`](#impl-stdi64--stdneg)
    - [`impl Std::I64 : Std::Rem`](#impl-stdi64--stdrem)
    - [`impl Std::I64 : Std::Sub`](#impl-stdi64--stdsub)
    - [`impl Std::I8 : Std::Add`](#impl-stdi8--stdadd)
    - [`impl Std::I8 : Std::Div`](#impl-stdi8--stddiv)
    - [`impl Std::I8 : Std::Eq`](#impl-stdi8--stdeq)
    - [`impl Std::I8 : Std::LessThan`](#impl-stdi8--stdlessthan)
    - [`impl Std::I8 : Std::LessThanOrEq`](#impl-stdi8--stdlessthanoreq)
    - [`impl Std::I8 : Std::Mul`](#impl-stdi8--stdmul)
    - [`impl Std::I8 : Std::Neg`](#impl-stdi8--stdneg)
    - [`impl Std::I8 : Std::Rem`](#impl-stdi8--stdrem)
    - [`impl Std::I8 : Std::Sub`](#impl-stdi8--stdsub)
    - [`impl Std::Ptr : Std::Eq`](#impl-stdptr--stdeq)
    - [`impl Std::U16 : Std::Add`](#impl-stdu16--stdadd)
    - [`impl Std::U16 : Std::Div`](#impl-stdu16--stddiv)
    - [`impl Std::U16 : Std::Eq`](#impl-stdu16--stdeq)
    - [`impl Std::U16 : Std::LessThan`](#impl-stdu16--stdlessthan)
    - [`impl Std::U16 : Std::LessThanOrEq`](#impl-stdu16--stdlessthanoreq)
    - [`impl Std::U16 : Std::Mul`](#impl-stdu16--stdmul)
    - [`impl Std::U16 : Std::Neg`](#impl-stdu16--stdneg)
    - [`impl Std::U16 : Std::Rem`](#impl-stdu16--stdrem)
    - [`impl Std::U16 : Std::Sub`](#impl-stdu16--stdsub)
    - [`impl Std::U32 : Std::Add`](#impl-stdu32--stdadd)
    - [`impl Std::U32 : Std::Div`](#impl-stdu32--stddiv)
    - [`impl Std::U32 : Std::Eq`](#impl-stdu32--stdeq)
    - [`impl Std::U32 : Std::LessThan`](#impl-stdu32--stdlessthan)
    - [`impl Std::U32 : Std::LessThanOrEq`](#impl-stdu32--stdlessthanoreq)
    - [`impl Std::U32 : Std::Mul`](#impl-stdu32--stdmul)
    - [`impl Std::U32 : Std::Neg`](#impl-stdu32--stdneg)
    - [`impl Std::U32 : Std::Rem`](#impl-stdu32--stdrem)
    - [`impl Std::U32 : Std::Sub`](#impl-stdu32--stdsub)
    - [`impl Std::U64 : Std::Add`](#impl-stdu64--stdadd)
    - [`impl Std::U64 : Std::Div`](#impl-stdu64--stddiv)
    - [`impl Std::U64 : Std::Eq`](#impl-stdu64--stdeq)
    - [`impl Std::U64 : Std::LessThan`](#impl-stdu64--stdlessthan)
    - [`impl Std::U64 : Std::LessThanOrEq`](#impl-stdu64--stdlessthanoreq)
    - [`impl Std::U64 : Std::Mul`](#impl-stdu64--stdmul)
    - [`impl Std::U64 : Std::Neg`](#impl-stdu64--stdneg)
    - [`impl Std::U64 : Std::Rem`](#impl-stdu64--stdrem)
    - [`impl Std::U64 : Std::Sub`](#impl-stdu64--stdsub)
    - [`impl Std::U8 : Std::Add`](#impl-stdu8--stdadd)
    - [`impl Std::U8 : Std::Div`](#impl-stdu8--stddiv)
    - [`impl Std::U8 : Std::Eq`](#impl-stdu8--stdeq)
    - [`impl Std::U8 : Std::LessThan`](#impl-stdu8--stdlessthan)
    - [`impl Std::U8 : Std::LessThanOrEq`](#impl-stdu8--stdlessthanoreq)
    - [`impl Std::U8 : Std::Mul`](#impl-stdu8--stdmul)
    - [`impl Std::U8 : Std::Neg`](#impl-stdu8--stdneg)
    - [`impl Std::U8 : Std::Rem`](#impl-stdu8--stdrem)
    - [`impl Std::U8 : Std::Sub`](#impl-stdu8--stdsub)
    - [`impl String : Add`](#impl-string--add)
    - [`impl String : Eq`](#impl-string--eq)
    - [`impl String : LessThan`](#impl-string--lessthan)
    - [`impl String : LessThanOrEq`](#impl-string--lessthanoreq)
    - [`impl String : ToString`](#impl-string--tostring)
    - [`impl U16 : FromBytes`](#impl-u16--frombytes)
    - [`impl U16 : FromString`](#impl-u16--fromstring)
    - [`impl U16 : ToBytes`](#impl-u16--tobytes)
    - [`impl U16 : ToString`](#impl-u16--tostring)
    - [`impl U16 : Zero`](#impl-u16--zero)
    - [`impl U32 : FromBytes`](#impl-u32--frombytes)
    - [`impl U32 : FromString`](#impl-u32--fromstring)
    - [`impl U32 : ToBytes`](#impl-u32--tobytes)
    - [`impl U32 : ToString`](#impl-u32--tostring)
    - [`impl U32 : Zero`](#impl-u32--zero)
    - [`impl U64 : FromBytes`](#impl-u64--frombytes)
    - [`impl U64 : FromString`](#impl-u64--fromstring)
    - [`impl U64 : ToBytes`](#impl-u64--tobytes)
    - [`impl U64 : ToString`](#impl-u64--tostring)
    - [`impl U64 : Zero`](#impl-u64--zero)
    - [`impl U8 : FromBytes`](#impl-u8--frombytes)
    - [`impl U8 : FromString`](#impl-u8--fromstring)
    - [`impl U8 : ToBytes`](#impl-u8--tobytes)
    - [`impl U8 : ToString`](#impl-u8--tostring)
    - [`impl U8 : Zero`](#impl-u8--zero)
- [Values](#values)
  - [`namespace Std`](#namespace-std-2)
    - [`compose : (a -> b) -> (b -> c) -> a -> c`](#compose--a---b---b---c---a---c)
    - [`fix : ((a -> b) -> a -> b) -> a -> b`](#fix--a---b---a---b---a---b)
    - [`loop : s -> (s -> Std::LoopResult s b) -> b`](#loop--s---s---stdloopresult-s-b---b)
    - [`loop_m : [m : Monad] s -> (s -> m (LoopResult s r)) -> m r`](#loop_m--m--monad-s---s---m-loopresult-s-r---m-r)
    - [`mark_threaded : a -> a`](#mark_threaded--a---a)
    - [`undefined : Std::Lazy a`](#undefined--stdlazy-a)
    - [`unsafe_is_unique : a -> (Std::Bool, a)`](#unsafe_is_unique--a---stdbool-a)
  - [`namespace Std::Array`](#namespace-stdarray)
    - [`@ : Std::I64 -> Std::Array a -> a`](#--stdi64---stdarray-a---a)
    - [`_get_ptr : Std::Array a -> Std::Ptr`](#_get_ptr--stdarray-a---stdptr)
    - [`_get_sub_size_asif : I64 -> I64 -> I64 -> I64 -> Array a -> Array a`](#_get_sub_size_asif--i64---i64---i64---i64---array-a---array-a)
    - [`_sort_range_using_buffer : Array a -> I64 -> I64 -> ((a, a) -> Bool) -> Array a -> (Array a, Array a)`](#_sort_range_using_buffer--array-a---i64---i64---a-a---bool---array-a---array-a-array-a)
    - [`_unsafe_get : Std::I64 -> Std::Array a -> a`](#_unsafe_get--stdi64---stdarray-a---a)
    - [`_unsafe_set : Std::I64 -> a -> Std::Array a -> Std::Array a`](#_unsafe_set--stdi64---a---stdarray-a---stdarray-a)
    - [`_unsafe_set_size : Std::I64 -> Std::Array a -> Std::Array a`](#_unsafe_set_size--stdi64---stdarray-a---stdarray-a)
    - [`act : [f : Functor] I64 -> (a -> f a) -> Array a -> f (Array a)`](#act--f--functor-i64---a---f-a---array-a---f-array-a)
    - [`append : Array a -> Array a -> Array a`](#append--array-a---array-a---array-a)
    - [`borrow_ptr : (Ptr -> b) -> Array a -> b`](#borrow_ptr--ptr---b---array-a---b)
    - [`empty : Std::I64 -> Std::Array a`](#empty--stdi64---stdarray-a)
    - [`fill : Std::I64 -> a -> Std::Array a`](#fill--stdi64---a---stdarray-a)
    - [`find_by : (a -> Bool) -> Array a -> Option I64`](#find_by--a---bool---array-a---option-i64)
    - [`force_unique : Std::Array a -> Std::Array a`](#force_unique--stdarray-a---stdarray-a)
    - [`from_iter : Iterator a -> Array a`](#from_iter--iterator-a---array-a)
    - [`from_map : I64 -> (I64 -> a) -> Array a`](#from_map--i64---i64---a---array-a)
    - [`get_capacity : Std::Array a -> Std::I64`](#get_capacity--stdarray-a---stdi64)
    - [`get_first : Array a -> Option a`](#get_first--array-a---option-a)
    - [`get_last : Array a -> Option a`](#get_last--array-a---option-a)
    - [`get_size : Std::Array a -> Std::I64`](#get_size--stdarray-a---stdi64)
    - [`get_sub : I64 -> I64 -> Array a -> Array a`](#get_sub--i64---i64---array-a---array-a)
    - [`is_empty : Array a -> Bool`](#is_empty--array-a---bool)
    - [`mod : Std::I64 -> (a -> a) -> Std::Array a -> Std::Array a`](#mod--stdi64---a---a---stdarray-a---stdarray-a)
    - [`pop_back : Array a -> Array a`](#pop_back--array-a---array-a)
    - [`push_back : a -> Array a -> Array a`](#push_back--a---array-a---array-a)
    - [`reserve : I64 -> Array a -> Array a`](#reserve--i64---array-a---array-a)
    - [`set : Std::I64 -> a -> Std::Array a -> Std::Array a`](#set--stdi64---a---stdarray-a---stdarray-a)
    - [`sort_by : ((a, a) -> Bool) -> Array a -> Array a`](#sort_by--a-a---bool---array-a---array-a)
    - [`to_iter : Array a -> Iterator a`](#to_iter--array-a---iterator-a)
    - [`truncate : I64 -> Array a -> Array a`](#truncate--i64---array-a---array-a)
  - [`namespace Std::F32`](#namespace-stdf32)
    - [`abs : F32 -> F32`](#abs--f32---f32)
    - [`infinity : Std::F32`](#infinity--stdf32)
    - [`quiet_nan : Std::F32`](#quiet_nan--stdf32)
    - [`to_CChar : Std::F32 -> Std::I8`](#to_cchar--stdf32---stdi8)
    - [`to_CDouble : Std::F32 -> Std::F64`](#to_cdouble--stdf32---stdf64)
    - [`to_CFloat : Std::F32 -> Std::F32`](#to_cfloat--stdf32---stdf32)
    - [`to_CInt : Std::F32 -> Std::I32`](#to_cint--stdf32---stdi32)
    - [`to_CLong : Std::F32 -> Std::I64`](#to_clong--stdf32---stdi64)
    - [`to_CLongLong : Std::F32 -> Std::I64`](#to_clonglong--stdf32---stdi64)
    - [`to_CShort : Std::F32 -> Std::I16`](#to_cshort--stdf32---stdi16)
    - [`to_CSizeT : Std::F32 -> Std::U64`](#to_csizet--stdf32---stdu64)
    - [`to_CUnsignedChar : Std::F32 -> Std::U8`](#to_cunsignedchar--stdf32---stdu8)
    - [`to_CUnsignedInt : Std::F32 -> Std::U32`](#to_cunsignedint--stdf32---stdu32)
    - [`to_CUnsignedLong : Std::F32 -> Std::U64`](#to_cunsignedlong--stdf32---stdu64)
    - [`to_CUnsignedLongLong : Std::F32 -> Std::U64`](#to_cunsignedlonglong--stdf32---stdu64)
    - [`to_CUnsignedShort : Std::F32 -> Std::U16`](#to_cunsignedshort--stdf32---stdu16)
    - [`to_F32 : Std::F32 -> Std::F32`](#to_f32--stdf32---stdf32)
    - [`to_F64 : Std::F32 -> Std::F64`](#to_f64--stdf32---stdf64)
    - [`to_I16 : Std::F32 -> Std::I16`](#to_i16--stdf32---stdi16)
    - [`to_I32 : Std::F32 -> Std::I32`](#to_i32--stdf32---stdi32)
    - [`to_I64 : Std::F32 -> Std::I64`](#to_i64--stdf32---stdi64)
    - [`to_I8 : Std::F32 -> Std::I8`](#to_i8--stdf32---stdi8)
    - [`to_U16 : Std::F32 -> Std::U16`](#to_u16--stdf32---stdu16)
    - [`to_U32 : Std::F32 -> Std::U32`](#to_u32--stdf32---stdu32)
    - [`to_U64 : Std::F32 -> Std::U64`](#to_u64--stdf32---stdu64)
    - [`to_U8 : Std::F32 -> Std::U8`](#to_u8--stdf32---stdu8)
    - [`to_string_exp : F32 -> String`](#to_string_exp--f32---string)
    - [`to_string_exp_precision : U8 -> F32 -> String`](#to_string_exp_precision--u8---f32---string)
    - [`to_string_precision : U8 -> F32 -> String`](#to_string_precision--u8---f32---string)
  - [`namespace Std::F64`](#namespace-stdf64)
    - [`abs : F64 -> F64`](#abs--f64---f64)
    - [`infinity : Std::F64`](#infinity--stdf64)
    - [`quiet_nan : Std::F64`](#quiet_nan--stdf64)
    - [`to_CChar : Std::F64 -> Std::I8`](#to_cchar--stdf64---stdi8)
    - [`to_CDouble : Std::F64 -> Std::F64`](#to_cdouble--stdf64---stdf64)
    - [`to_CFloat : Std::F64 -> Std::F32`](#to_cfloat--stdf64---stdf32)
    - [`to_CInt : Std::F64 -> Std::I32`](#to_cint--stdf64---stdi32)
    - [`to_CLong : Std::F64 -> Std::I64`](#to_clong--stdf64---stdi64)
    - [`to_CLongLong : Std::F64 -> Std::I64`](#to_clonglong--stdf64---stdi64)
    - [`to_CShort : Std::F64 -> Std::I16`](#to_cshort--stdf64---stdi16)
    - [`to_CSizeT : Std::F64 -> Std::U64`](#to_csizet--stdf64---stdu64)
    - [`to_CUnsignedChar : Std::F64 -> Std::U8`](#to_cunsignedchar--stdf64---stdu8)
    - [`to_CUnsignedInt : Std::F64 -> Std::U32`](#to_cunsignedint--stdf64---stdu32)
    - [`to_CUnsignedLong : Std::F64 -> Std::U64`](#to_cunsignedlong--stdf64---stdu64)
    - [`to_CUnsignedLongLong : Std::F64 -> Std::U64`](#to_cunsignedlonglong--stdf64---stdu64)
    - [`to_CUnsignedShort : Std::F64 -> Std::U16`](#to_cunsignedshort--stdf64---stdu16)
    - [`to_F32 : Std::F64 -> Std::F32`](#to_f32--stdf64---stdf32)
    - [`to_F64 : Std::F64 -> Std::F64`](#to_f64--stdf64---stdf64)
    - [`to_I16 : Std::F64 -> Std::I16`](#to_i16--stdf64---stdi16)
    - [`to_I32 : Std::F64 -> Std::I32`](#to_i32--stdf64---stdi32)
    - [`to_I64 : Std::F64 -> Std::I64`](#to_i64--stdf64---stdi64)
    - [`to_I8 : Std::F64 -> Std::I8`](#to_i8--stdf64---stdi8)
    - [`to_U16 : Std::F64 -> Std::U16`](#to_u16--stdf64---stdu16)
    - [`to_U32 : Std::F64 -> Std::U32`](#to_u32--stdf64---stdu32)
    - [`to_U64 : Std::F64 -> Std::U64`](#to_u64--stdf64---stdu64)
    - [`to_U8 : Std::F64 -> Std::U8`](#to_u8--stdf64---stdu8)
    - [`to_string_exp : F64 -> String`](#to_string_exp--f64---string)
    - [`to_string_exp_precision : U8 -> F64 -> String`](#to_string_exp_precision--u8---f64---string)
    - [`to_string_precision : U8 -> F64 -> String`](#to_string_precision--u8---f64---string)
  - [`namespace Std::FFI`](#namespace-stdffi-1)
    - [`_unsafe_get_boxed_data_ptr : a -> Std::Ptr`](#_unsafe_get_boxed_data_ptr--a---stdptr)
    - [`unsafe_borrow_boxed_data_ptr : (Ptr -> b) -> a -> b`](#unsafe_borrow_boxed_data_ptr--ptr---b---a---b)
    - [`unsafe_clear_errno : () -> ()`](#unsafe_clear_errno-----)
    - [`unsafe_get_boxed_value_from_retained_ptr : Std::Ptr -> a`](#unsafe_get_boxed_value_from_retained_ptr--stdptr---a)
    - [`unsafe_get_errno : () -> CInt`](#unsafe_get_errno-----cint)
    - [`unsafe_get_release_function_of_boxed_value : Std::Lazy a -> Std::Ptr`](#unsafe_get_release_function_of_boxed_value--stdlazy-a---stdptr)
    - [`unsafe_get_retain_function_of_boxed_value : Std::Lazy a -> Std::Ptr`](#unsafe_get_retain_function_of_boxed_value--stdlazy-a---stdptr)
    - [`unsafe_get_retained_ptr_of_boxed_value : a -> Std::Ptr`](#unsafe_get_retained_ptr_of_boxed_value--a---stdptr)
  - [`namespace Std::FFI::Destructor`](#namespace-stdffidestructor)
    - [`borrow : (a -> b) -> Destructor a -> b`](#borrow--a---b---destructor-a---b)
    - [`make : a -> (a -> ()) -> Destructor a`](#make--a---a------destructor-a)
  - [`namespace Std::Functor`](#namespace-stdfunctor)
    - [`forget : [f : Functor] f a -> f ()`](#forget--f--functor-f-a---f-)
  - [`namespace Std::I16`](#namespace-stdi16)
    - [`abs : I16 -> I16`](#abs--i16---i16)
    - [`bit_and : Std::I16 -> Std::I16 -> Std::I16`](#bit_and--stdi16---stdi16---stdi16)
    - [`bit_or : Std::I16 -> Std::I16 -> Std::I16`](#bit_or--stdi16---stdi16---stdi16)
    - [`bit_xor : Std::I16 -> Std::I16 -> Std::I16`](#bit_xor--stdi16---stdi16---stdi16)
    - [`maximum : I16`](#maximum--i16)
    - [`minimum : I16`](#minimum--i16)
    - [`shift_left : Std::I16 -> Std::I16 -> Std::I16`](#shift_left--stdi16---stdi16---stdi16)
    - [`shift_right : Std::I16 -> Std::I16 -> Std::I16`](#shift_right--stdi16---stdi16---stdi16)
    - [`to_CChar : Std::I16 -> Std::I8`](#to_cchar--stdi16---stdi8)
    - [`to_CDouble : Std::I16 -> Std::F64`](#to_cdouble--stdi16---stdf64)
    - [`to_CFloat : Std::I16 -> Std::F32`](#to_cfloat--stdi16---stdf32)
    - [`to_CInt : Std::I16 -> Std::I32`](#to_cint--stdi16---stdi32)
    - [`to_CLong : Std::I16 -> Std::I64`](#to_clong--stdi16---stdi64)
    - [`to_CLongLong : Std::I16 -> Std::I64`](#to_clonglong--stdi16---stdi64)
    - [`to_CShort : Std::I16 -> Std::I16`](#to_cshort--stdi16---stdi16)
    - [`to_CSizeT : Std::I16 -> Std::U64`](#to_csizet--stdi16---stdu64)
    - [`to_CUnsignedChar : Std::I16 -> Std::U8`](#to_cunsignedchar--stdi16---stdu8)
    - [`to_CUnsignedInt : Std::I16 -> Std::U32`](#to_cunsignedint--stdi16---stdu32)
    - [`to_CUnsignedLong : Std::I16 -> Std::U64`](#to_cunsignedlong--stdi16---stdu64)
    - [`to_CUnsignedLongLong : Std::I16 -> Std::U64`](#to_cunsignedlonglong--stdi16---stdu64)
    - [`to_CUnsignedShort : Std::I16 -> Std::U16`](#to_cunsignedshort--stdi16---stdu16)
    - [`to_F32 : Std::I16 -> Std::F32`](#to_f32--stdi16---stdf32)
    - [`to_F64 : Std::I16 -> Std::F64`](#to_f64--stdi16---stdf64)
    - [`to_I16 : Std::I16 -> Std::I16`](#to_i16--stdi16---stdi16)
    - [`to_I32 : Std::I16 -> Std::I32`](#to_i32--stdi16---stdi32)
    - [`to_I64 : Std::I16 -> Std::I64`](#to_i64--stdi16---stdi64)
    - [`to_I8 : Std::I16 -> Std::I8`](#to_i8--stdi16---stdi8)
    - [`to_U16 : Std::I16 -> Std::U16`](#to_u16--stdi16---stdu16)
    - [`to_U32 : Std::I16 -> Std::U32`](#to_u32--stdi16---stdu32)
    - [`to_U64 : Std::I16 -> Std::U64`](#to_u64--stdi16---stdu64)
    - [`to_U8 : Std::I16 -> Std::U8`](#to_u8--stdi16---stdu8)
  - [`namespace Std::I32`](#namespace-stdi32)
    - [`abs : I32 -> I32`](#abs--i32---i32)
    - [`bit_and : Std::I32 -> Std::I32 -> Std::I32`](#bit_and--stdi32---stdi32---stdi32)
    - [`bit_or : Std::I32 -> Std::I32 -> Std::I32`](#bit_or--stdi32---stdi32---stdi32)
    - [`bit_xor : Std::I32 -> Std::I32 -> Std::I32`](#bit_xor--stdi32---stdi32---stdi32)
    - [`maximum : I32`](#maximum--i32)
    - [`minimum : I32`](#minimum--i32)
    - [`shift_left : Std::I32 -> Std::I32 -> Std::I32`](#shift_left--stdi32---stdi32---stdi32)
    - [`shift_right : Std::I32 -> Std::I32 -> Std::I32`](#shift_right--stdi32---stdi32---stdi32)
    - [`to_CChar : Std::I32 -> Std::I8`](#to_cchar--stdi32---stdi8)
    - [`to_CDouble : Std::I32 -> Std::F64`](#to_cdouble--stdi32---stdf64)
    - [`to_CFloat : Std::I32 -> Std::F32`](#to_cfloat--stdi32---stdf32)
    - [`to_CInt : Std::I32 -> Std::I32`](#to_cint--stdi32---stdi32)
    - [`to_CLong : Std::I32 -> Std::I64`](#to_clong--stdi32---stdi64)
    - [`to_CLongLong : Std::I32 -> Std::I64`](#to_clonglong--stdi32---stdi64)
    - [`to_CShort : Std::I32 -> Std::I16`](#to_cshort--stdi32---stdi16)
    - [`to_CSizeT : Std::I32 -> Std::U64`](#to_csizet--stdi32---stdu64)
    - [`to_CUnsignedChar : Std::I32 -> Std::U8`](#to_cunsignedchar--stdi32---stdu8)
    - [`to_CUnsignedInt : Std::I32 -> Std::U32`](#to_cunsignedint--stdi32---stdu32)
    - [`to_CUnsignedLong : Std::I32 -> Std::U64`](#to_cunsignedlong--stdi32---stdu64)
    - [`to_CUnsignedLongLong : Std::I32 -> Std::U64`](#to_cunsignedlonglong--stdi32---stdu64)
    - [`to_CUnsignedShort : Std::I32 -> Std::U16`](#to_cunsignedshort--stdi32---stdu16)
    - [`to_F32 : Std::I32 -> Std::F32`](#to_f32--stdi32---stdf32)
    - [`to_F64 : Std::I32 -> Std::F64`](#to_f64--stdi32---stdf64)
    - [`to_I16 : Std::I32 -> Std::I16`](#to_i16--stdi32---stdi16)
    - [`to_I32 : Std::I32 -> Std::I32`](#to_i32--stdi32---stdi32)
    - [`to_I64 : Std::I32 -> Std::I64`](#to_i64--stdi32---stdi64)
    - [`to_I8 : Std::I32 -> Std::I8`](#to_i8--stdi32---stdi8)
    - [`to_U16 : Std::I32 -> Std::U16`](#to_u16--stdi32---stdu16)
    - [`to_U32 : Std::I32 -> Std::U32`](#to_u32--stdi32---stdu32)
    - [`to_U64 : Std::I32 -> Std::U64`](#to_u64--stdi32---stdu64)
    - [`to_U8 : Std::I32 -> Std::U8`](#to_u8--stdi32---stdu8)
  - [`namespace Std::I64`](#namespace-stdi64)
    - [`abs : I64 -> I64`](#abs--i64---i64)
    - [`bit_and : Std::I64 -> Std::I64 -> Std::I64`](#bit_and--stdi64---stdi64---stdi64)
    - [`bit_or : Std::I64 -> Std::I64 -> Std::I64`](#bit_or--stdi64---stdi64---stdi64)
    - [`bit_xor : Std::I64 -> Std::I64 -> Std::I64`](#bit_xor--stdi64---stdi64---stdi64)
    - [`maximum : I64`](#maximum--i64)
    - [`minimum : I64`](#minimum--i64)
    - [`shift_left : Std::I64 -> Std::I64 -> Std::I64`](#shift_left--stdi64---stdi64---stdi64)
    - [`shift_right : Std::I64 -> Std::I64 -> Std::I64`](#shift_right--stdi64---stdi64---stdi64)
    - [`to_CChar : Std::I64 -> Std::I8`](#to_cchar--stdi64---stdi8)
    - [`to_CDouble : Std::I64 -> Std::F64`](#to_cdouble--stdi64---stdf64)
    - [`to_CFloat : Std::I64 -> Std::F32`](#to_cfloat--stdi64---stdf32)
    - [`to_CInt : Std::I64 -> Std::I32`](#to_cint--stdi64---stdi32)
    - [`to_CLong : Std::I64 -> Std::I64`](#to_clong--stdi64---stdi64)
    - [`to_CLongLong : Std::I64 -> Std::I64`](#to_clonglong--stdi64---stdi64)
    - [`to_CShort : Std::I64 -> Std::I16`](#to_cshort--stdi64---stdi16)
    - [`to_CSizeT : Std::I64 -> Std::U64`](#to_csizet--stdi64---stdu64)
    - [`to_CUnsignedChar : Std::I64 -> Std::U8`](#to_cunsignedchar--stdi64---stdu8)
    - [`to_CUnsignedInt : Std::I64 -> Std::U32`](#to_cunsignedint--stdi64---stdu32)
    - [`to_CUnsignedLong : Std::I64 -> Std::U64`](#to_cunsignedlong--stdi64---stdu64)
    - [`to_CUnsignedLongLong : Std::I64 -> Std::U64`](#to_cunsignedlonglong--stdi64---stdu64)
    - [`to_CUnsignedShort : Std::I64 -> Std::U16`](#to_cunsignedshort--stdi64---stdu16)
    - [`to_F32 : Std::I64 -> Std::F32`](#to_f32--stdi64---stdf32)
    - [`to_F64 : Std::I64 -> Std::F64`](#to_f64--stdi64---stdf64)
    - [`to_I16 : Std::I64 -> Std::I16`](#to_i16--stdi64---stdi16)
    - [`to_I32 : Std::I64 -> Std::I32`](#to_i32--stdi64---stdi32)
    - [`to_I64 : Std::I64 -> Std::I64`](#to_i64--stdi64---stdi64)
    - [`to_I8 : Std::I64 -> Std::I8`](#to_i8--stdi64---stdi8)
    - [`to_U16 : Std::I64 -> Std::U16`](#to_u16--stdi64---stdu16)
    - [`to_U32 : Std::I64 -> Std::U32`](#to_u32--stdi64---stdu32)
    - [`to_U64 : Std::I64 -> Std::U64`](#to_u64--stdi64---stdu64)
    - [`to_U8 : Std::I64 -> Std::U8`](#to_u8--stdi64---stdu8)
  - [`namespace Std::I8`](#namespace-stdi8)
    - [`abs : I8 -> I8`](#abs--i8---i8)
    - [`bit_and : Std::I8 -> Std::I8 -> Std::I8`](#bit_and--stdi8---stdi8---stdi8)
    - [`bit_or : Std::I8 -> Std::I8 -> Std::I8`](#bit_or--stdi8---stdi8---stdi8)
    - [`bit_xor : Std::I8 -> Std::I8 -> Std::I8`](#bit_xor--stdi8---stdi8---stdi8)
    - [`maximum : I8`](#maximum--i8)
    - [`minimum : I8`](#minimum--i8)
    - [`shift_left : Std::I8 -> Std::I8 -> Std::I8`](#shift_left--stdi8---stdi8---stdi8)
    - [`shift_right : Std::I8 -> Std::I8 -> Std::I8`](#shift_right--stdi8---stdi8---stdi8)
    - [`to_CChar : Std::I8 -> Std::I8`](#to_cchar--stdi8---stdi8)
    - [`to_CDouble : Std::I8 -> Std::F64`](#to_cdouble--stdi8---stdf64)
    - [`to_CFloat : Std::I8 -> Std::F32`](#to_cfloat--stdi8---stdf32)
    - [`to_CInt : Std::I8 -> Std::I32`](#to_cint--stdi8---stdi32)
    - [`to_CLong : Std::I8 -> Std::I64`](#to_clong--stdi8---stdi64)
    - [`to_CLongLong : Std::I8 -> Std::I64`](#to_clonglong--stdi8---stdi64)
    - [`to_CShort : Std::I8 -> Std::I16`](#to_cshort--stdi8---stdi16)
    - [`to_CSizeT : Std::I8 -> Std::U64`](#to_csizet--stdi8---stdu64)
    - [`to_CUnsignedChar : Std::I8 -> Std::U8`](#to_cunsignedchar--stdi8---stdu8)
    - [`to_CUnsignedInt : Std::I8 -> Std::U32`](#to_cunsignedint--stdi8---stdu32)
    - [`to_CUnsignedLong : Std::I8 -> Std::U64`](#to_cunsignedlong--stdi8---stdu64)
    - [`to_CUnsignedLongLong : Std::I8 -> Std::U64`](#to_cunsignedlonglong--stdi8---stdu64)
    - [`to_CUnsignedShort : Std::I8 -> Std::U16`](#to_cunsignedshort--stdi8---stdu16)
    - [`to_F32 : Std::I8 -> Std::F32`](#to_f32--stdi8---stdf32)
    - [`to_F64 : Std::I8 -> Std::F64`](#to_f64--stdi8---stdf64)
    - [`to_I16 : Std::I8 -> Std::I16`](#to_i16--stdi8---stdi16)
    - [`to_I32 : Std::I8 -> Std::I32`](#to_i32--stdi8---stdi32)
    - [`to_I64 : Std::I8 -> Std::I64`](#to_i64--stdi8---stdi64)
    - [`to_I8 : Std::I8 -> Std::I8`](#to_i8--stdi8---stdi8)
    - [`to_U16 : Std::I8 -> Std::U16`](#to_u16--stdi8---stdu16)
    - [`to_U32 : Std::I8 -> Std::U32`](#to_u32--stdi8---stdu32)
    - [`to_U64 : Std::I8 -> Std::U64`](#to_u64--stdi8---stdu64)
    - [`to_U8 : Std::I8 -> Std::U8`](#to_u8--stdi8---stdu8)
  - [`namespace Std::IO`](#namespace-stdio-1)
    - [`_read_line_inner : Bool -> IOHandle -> IOFail String`](#_read_line_inner--bool---iohandle---iofail-string)
    - [`_unsafe_perform : IO a -> a`](#_unsafe_perform--io-a---a)
    - [`close_file : IOHandle -> IO ()`](#close_file--iohandle---io-)
    - [`eprint : String -> IO ()`](#eprint--string---io-)
    - [`eprintln : String -> IO ()`](#eprintln--string---io-)
    - [`exit : I64 -> IO a`](#exit--i64---io-a)
    - [`exit_with_msg : I64 -> String -> IO a`](#exit_with_msg--i64---string---io-a)
    - [`from_func : (() -> a) -> IO a`](#from_func-----a---io-a)
    - [`get_arg : I64 -> IO (Option String)`](#get_arg--i64---io-option-string)
    - [`get_arg_count : IO I64`](#get_arg_count--io-i64)
    - [`get_args : IO (Array String)`](#get_args--io-array-string)
    - [`input_line : IO String`](#input_line--io-string)
    - [`is_eof : IOHandle -> IO Bool`](#is_eof--iohandle---io-bool)
    - [`loop_lines : IOHandle -> s -> (s -> String -> LoopResult s s) -> IOFail s`](#loop_lines--iohandle---s---s---string---loopresult-s-s---iofail-s)
    - [`loop_lines_io : IOHandle -> s -> (s -> String -> IOFail (LoopResult s s)) -> IOFail s`](#loop_lines_io--iohandle---s---s---string---iofail-loopresult-s-s---iofail-s)
    - [`open_file : Path -> String -> IOFail IOHandle`](#open_file--path---string---iofail-iohandle)
    - [`print : String -> IO ()`](#print--string---io-)
    - [`println : String -> IO ()`](#println--string---io-)
    - [`read_bytes : IOHandle -> IOFail (Array U8)`](#read_bytes--iohandle---iofail-array-u8)
    - [`read_file_bytes : Path -> IOFail (Array U8)`](#read_file_bytes--path---iofail-array-u8)
    - [`read_file_string : Path -> IOFail String`](#read_file_string--path---iofail-string)
    - [`read_line : IOHandle -> IOFail String`](#read_line--iohandle---iofail-string)
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
  - [`namespace Std::IO::IOFail`](#namespace-stdioiofail)
    - [`from_result : Result ErrMsg a -> IOFail a`](#from_result--result-errmsg-a---iofail-a)
    - [`lift : IO a -> IOFail a`](#lift--io-a---iofail-a)
    - [`throw : ErrMsg -> IOFail a`](#throw--errmsg---iofail-a)
    - [`to_result : IOFail a -> IO (Result ErrMsg a)`](#to_result--iofail-a---io-result-errmsg-a)
    - [`try : (ErrMsg -> IO a) -> IOFail a -> IO a`](#try--errmsg---io-a---iofail-a---io-a)
  - [`namespace Std::IO::IOHandle`](#namespace-stdioiohandle)
    - [`_file_ptr : IOHandle -> Ptr`](#_file_ptr--iohandle---ptr)
    - [`_unsafe_close : IOHandle -> ()`](#_unsafe_close--iohandle---)
    - [`from_file_ptr : Ptr -> IOHandle`](#from_file_ptr--ptr---iohandle)
  - [`namespace Std::Iterator`](#namespace-stditerator)
    - [`_flatten : Iterator (Iterator a) -> Iterator a`](#_flatten--iterator-iterator-a---iterator-a)
    - [`_flatten_sub : Iterator a -> Iterator (Iterator a) -> Iterator a`](#_flatten_sub--iterator-a---iterator-iterator-a---iterator-a)
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
    - [`product : Iterator a -> Iterator b -> Iterator (b, a)`](#product--iterator-a---iterator-b---iterator-b-a)
    - [`push_front : a -> Iterator a -> Iterator a`](#push_front--a---iterator-a---iterator-a)
    - [`range : I64 -> I64 -> Iterator I64`](#range--i64---i64---iterator-i64)
    - [`reverse : Iterator a -> Iterator a`](#reverse--iterator-a---iterator-a)
    - [`subsequences : Iterator a -> Iterator (Iterator a)`](#subsequences--iterator-a---iterator-iterator-a)
    - [`sum : [a : Additive] Iterator a -> a`](#sum--a--additive-iterator-a---a)
    - [`take : I64 -> Iterator a -> Iterator a`](#take--i64---iterator-a---iterator-a)
    - [`take_while : (a -> Bool) -> Iterator a -> Iterator a`](#take_while--a---bool---iterator-a---iterator-a)
    - [`to_array : Iterator a -> Array a`](#to_array--iterator-a---array-a)
    - [`zip : Iterator b -> Iterator a -> Iterator (a, b)`](#zip--iterator-b---iterator-a---iterator-a-b)
  - [`namespace Std::LessThan`](#namespace-stdlessthan)
    - [`max : [a : LessThan] a -> a -> a`](#max--a--lessthan-a---a---a)
    - [`min : [a : LessThan] a -> a -> a`](#min--a--lessthan-a---a---a)
  - [`namespace Std::LoopResult`](#namespace-stdloopresult)
    - [`break_m : [m : Monad] r -> m (LoopResult s r)`](#break_m--m--monad-r---m-loopresult-s-r)
    - [`continue_m : [m : Monad] s -> m (LoopResult s r)`](#continue_m--m--monad-s---m-loopresult-s-r)
  - [`namespace Std::Monad`](#namespace-stdmonad)
    - [`flatten : [m : Monad] m (m a) -> m a`](#flatten--m--monad-m-m-a---m-a)
    - [`unless : [m : Monad] Bool -> m () -> m ()`](#unless--m--monad-bool---m----m-)
    - [`when : [m : Monad] Bool -> m () -> m ()`](#when--m--monad-bool---m----m-)
  - [`namespace Std::Option`](#namespace-stdoption)
    - [`as_some_or : a -> Option a -> a`](#as_some_or--a---option-a---a)
    - [`map_or : b -> (a -> b) -> Option a -> b`](#map_or--b---a---b---option-a---b)
  - [`namespace Std::Path`](#namespace-stdpath)
    - [`parse : String -> Option Path`](#parse--string---option-path)
  - [`namespace Std::Ptr`](#namespace-stdptr)
    - [`add_offset : I64 -> Ptr -> Ptr`](#add_offset--i64---ptr---ptr)
    - [`subtract_ptr : Ptr -> Ptr -> I64`](#subtract_ptr--ptr---ptr---i64)
  - [`namespace Std::PunchedArray`](#namespace-stdpunchedarray)
    - [`plug_in : a -> PunchedArray a -> Array a`](#plug_in--a---punchedarray-a---array-a)
    - [`unsafe_punch : I64 -> Array a -> (PunchedArray a, a)`](#unsafe_punch--i64---array-a---punchedarray-a-a)
  - [`namespace Std::Result`](#namespace-stdresult)
    - [`unwrap : Result e o -> o`](#unwrap--result-e-o---o)
  - [`namespace Std::String`](#namespace-stdstring)
    - [`_get_c_str : String -> Ptr`](#_get_c_str--string---ptr)
    - [`_unsafe_from_c_str : Array U8 -> String`](#_unsafe_from_c_str--array-u8---string)
    - [`_unsafe_from_c_str_ptr : Ptr -> String`](#_unsafe_from_c_str_ptr--ptr---string)
    - [`borrow_c_str : (Ptr -> a) -> String -> a`](#borrow_c_str--ptr---a---string---a)
    - [`concat : String -> String -> String`](#concat--string---string---string)
    - [`concat_iter : Iterator String -> String`](#concat_iter--iterator-string---string)
    - [`empty : I64 -> String`](#empty--i64---string)
    - [`find : String -> I64 -> String -> Option I64`](#find--string---i64---string---option-i64)
    - [`get_bytes : String -> Array U8`](#get_bytes--string---array-u8)
    - [`get_first_byte : String -> Option U8`](#get_first_byte--string---option-u8)
    - [`get_last_byte : String -> Option U8`](#get_last_byte--string---option-u8)
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
  - [`namespace Std::U16`](#namespace-stdu16)
    - [`bit_and : Std::U16 -> Std::U16 -> Std::U16`](#bit_and--stdu16---stdu16---stdu16)
    - [`bit_or : Std::U16 -> Std::U16 -> Std::U16`](#bit_or--stdu16---stdu16---stdu16)
    - [`bit_xor : Std::U16 -> Std::U16 -> Std::U16`](#bit_xor--stdu16---stdu16---stdu16)
    - [`maximum : U16`](#maximum--u16)
    - [`minimum : U16`](#minimum--u16)
    - [`shift_left : Std::U16 -> Std::U16 -> Std::U16`](#shift_left--stdu16---stdu16---stdu16)
    - [`shift_right : Std::U16 -> Std::U16 -> Std::U16`](#shift_right--stdu16---stdu16---stdu16)
    - [`to_CChar : Std::U16 -> Std::I8`](#to_cchar--stdu16---stdi8)
    - [`to_CDouble : Std::U16 -> Std::F64`](#to_cdouble--stdu16---stdf64)
    - [`to_CFloat : Std::U16 -> Std::F32`](#to_cfloat--stdu16---stdf32)
    - [`to_CInt : Std::U16 -> Std::I32`](#to_cint--stdu16---stdi32)
    - [`to_CLong : Std::U16 -> Std::I64`](#to_clong--stdu16---stdi64)
    - [`to_CLongLong : Std::U16 -> Std::I64`](#to_clonglong--stdu16---stdi64)
    - [`to_CShort : Std::U16 -> Std::I16`](#to_cshort--stdu16---stdi16)
    - [`to_CSizeT : Std::U16 -> Std::U64`](#to_csizet--stdu16---stdu64)
    - [`to_CUnsignedChar : Std::U16 -> Std::U8`](#to_cunsignedchar--stdu16---stdu8)
    - [`to_CUnsignedInt : Std::U16 -> Std::U32`](#to_cunsignedint--stdu16---stdu32)
    - [`to_CUnsignedLong : Std::U16 -> Std::U64`](#to_cunsignedlong--stdu16---stdu64)
    - [`to_CUnsignedLongLong : Std::U16 -> Std::U64`](#to_cunsignedlonglong--stdu16---stdu64)
    - [`to_CUnsignedShort : Std::U16 -> Std::U16`](#to_cunsignedshort--stdu16---stdu16)
    - [`to_F32 : Std::U16 -> Std::F32`](#to_f32--stdu16---stdf32)
    - [`to_F64 : Std::U16 -> Std::F64`](#to_f64--stdu16---stdf64)
    - [`to_I16 : Std::U16 -> Std::I16`](#to_i16--stdu16---stdi16)
    - [`to_I32 : Std::U16 -> Std::I32`](#to_i32--stdu16---stdi32)
    - [`to_I64 : Std::U16 -> Std::I64`](#to_i64--stdu16---stdi64)
    - [`to_I8 : Std::U16 -> Std::I8`](#to_i8--stdu16---stdi8)
    - [`to_U16 : Std::U16 -> Std::U16`](#to_u16--stdu16---stdu16)
    - [`to_U32 : Std::U16 -> Std::U32`](#to_u32--stdu16---stdu32)
    - [`to_U64 : Std::U16 -> Std::U64`](#to_u64--stdu16---stdu64)
    - [`to_U8 : Std::U16 -> Std::U8`](#to_u8--stdu16---stdu8)
  - [`namespace Std::U32`](#namespace-stdu32)
    - [`bit_and : Std::U32 -> Std::U32 -> Std::U32`](#bit_and--stdu32---stdu32---stdu32)
    - [`bit_or : Std::U32 -> Std::U32 -> Std::U32`](#bit_or--stdu32---stdu32---stdu32)
    - [`bit_xor : Std::U32 -> Std::U32 -> Std::U32`](#bit_xor--stdu32---stdu32---stdu32)
    - [`maximum : U32`](#maximum--u32)
    - [`minimum : U32`](#minimum--u32)
    - [`shift_left : Std::U32 -> Std::U32 -> Std::U32`](#shift_left--stdu32---stdu32---stdu32)
    - [`shift_right : Std::U32 -> Std::U32 -> Std::U32`](#shift_right--stdu32---stdu32---stdu32)
    - [`to_CChar : Std::U32 -> Std::I8`](#to_cchar--stdu32---stdi8)
    - [`to_CDouble : Std::U32 -> Std::F64`](#to_cdouble--stdu32---stdf64)
    - [`to_CFloat : Std::U32 -> Std::F32`](#to_cfloat--stdu32---stdf32)
    - [`to_CInt : Std::U32 -> Std::I32`](#to_cint--stdu32---stdi32)
    - [`to_CLong : Std::U32 -> Std::I64`](#to_clong--stdu32---stdi64)
    - [`to_CLongLong : Std::U32 -> Std::I64`](#to_clonglong--stdu32---stdi64)
    - [`to_CShort : Std::U32 -> Std::I16`](#to_cshort--stdu32---stdi16)
    - [`to_CSizeT : Std::U32 -> Std::U64`](#to_csizet--stdu32---stdu64)
    - [`to_CUnsignedChar : Std::U32 -> Std::U8`](#to_cunsignedchar--stdu32---stdu8)
    - [`to_CUnsignedInt : Std::U32 -> Std::U32`](#to_cunsignedint--stdu32---stdu32)
    - [`to_CUnsignedLong : Std::U32 -> Std::U64`](#to_cunsignedlong--stdu32---stdu64)
    - [`to_CUnsignedLongLong : Std::U32 -> Std::U64`](#to_cunsignedlonglong--stdu32---stdu64)
    - [`to_CUnsignedShort : Std::U32 -> Std::U16`](#to_cunsignedshort--stdu32---stdu16)
    - [`to_F32 : Std::U32 -> Std::F32`](#to_f32--stdu32---stdf32)
    - [`to_F64 : Std::U32 -> Std::F64`](#to_f64--stdu32---stdf64)
    - [`to_I16 : Std::U32 -> Std::I16`](#to_i16--stdu32---stdi16)
    - [`to_I32 : Std::U32 -> Std::I32`](#to_i32--stdu32---stdi32)
    - [`to_I64 : Std::U32 -> Std::I64`](#to_i64--stdu32---stdi64)
    - [`to_I8 : Std::U32 -> Std::I8`](#to_i8--stdu32---stdi8)
    - [`to_U16 : Std::U32 -> Std::U16`](#to_u16--stdu32---stdu16)
    - [`to_U32 : Std::U32 -> Std::U32`](#to_u32--stdu32---stdu32)
    - [`to_U64 : Std::U32 -> Std::U64`](#to_u64--stdu32---stdu64)
    - [`to_U8 : Std::U32 -> Std::U8`](#to_u8--stdu32---stdu8)
  - [`namespace Std::U64`](#namespace-stdu64)
    - [`bit_and : Std::U64 -> Std::U64 -> Std::U64`](#bit_and--stdu64---stdu64---stdu64)
    - [`bit_or : Std::U64 -> Std::U64 -> Std::U64`](#bit_or--stdu64---stdu64---stdu64)
    - [`bit_xor : Std::U64 -> Std::U64 -> Std::U64`](#bit_xor--stdu64---stdu64---stdu64)
    - [`maximum : U64`](#maximum--u64)
    - [`minimum : U64`](#minimum--u64)
    - [`shift_left : Std::U64 -> Std::U64 -> Std::U64`](#shift_left--stdu64---stdu64---stdu64)
    - [`shift_right : Std::U64 -> Std::U64 -> Std::U64`](#shift_right--stdu64---stdu64---stdu64)
    - [`to_CChar : Std::U64 -> Std::I8`](#to_cchar--stdu64---stdi8)
    - [`to_CDouble : Std::U64 -> Std::F64`](#to_cdouble--stdu64---stdf64)
    - [`to_CFloat : Std::U64 -> Std::F32`](#to_cfloat--stdu64---stdf32)
    - [`to_CInt : Std::U64 -> Std::I32`](#to_cint--stdu64---stdi32)
    - [`to_CLong : Std::U64 -> Std::I64`](#to_clong--stdu64---stdi64)
    - [`to_CLongLong : Std::U64 -> Std::I64`](#to_clonglong--stdu64---stdi64)
    - [`to_CShort : Std::U64 -> Std::I16`](#to_cshort--stdu64---stdi16)
    - [`to_CSizeT : Std::U64 -> Std::U64`](#to_csizet--stdu64---stdu64)
    - [`to_CUnsignedChar : Std::U64 -> Std::U8`](#to_cunsignedchar--stdu64---stdu8)
    - [`to_CUnsignedInt : Std::U64 -> Std::U32`](#to_cunsignedint--stdu64---stdu32)
    - [`to_CUnsignedLong : Std::U64 -> Std::U64`](#to_cunsignedlong--stdu64---stdu64)
    - [`to_CUnsignedLongLong : Std::U64 -> Std::U64`](#to_cunsignedlonglong--stdu64---stdu64)
    - [`to_CUnsignedShort : Std::U64 -> Std::U16`](#to_cunsignedshort--stdu64---stdu16)
    - [`to_F32 : Std::U64 -> Std::F32`](#to_f32--stdu64---stdf32)
    - [`to_F64 : Std::U64 -> Std::F64`](#to_f64--stdu64---stdf64)
    - [`to_I16 : Std::U64 -> Std::I16`](#to_i16--stdu64---stdi16)
    - [`to_I32 : Std::U64 -> Std::I32`](#to_i32--stdu64---stdi32)
    - [`to_I64 : Std::U64 -> Std::I64`](#to_i64--stdu64---stdi64)
    - [`to_I8 : Std::U64 -> Std::I8`](#to_i8--stdu64---stdi8)
    - [`to_U16 : Std::U64 -> Std::U16`](#to_u16--stdu64---stdu16)
    - [`to_U32 : Std::U64 -> Std::U32`](#to_u32--stdu64---stdu32)
    - [`to_U64 : Std::U64 -> Std::U64`](#to_u64--stdu64---stdu64)
    - [`to_U8 : Std::U64 -> Std::U8`](#to_u8--stdu64---stdu8)
  - [`namespace Std::U8`](#namespace-stdu8)
    - [`bit_and : Std::U8 -> Std::U8 -> Std::U8`](#bit_and--stdu8---stdu8---stdu8)
    - [`bit_or : Std::U8 -> Std::U8 -> Std::U8`](#bit_or--stdu8---stdu8---stdu8)
    - [`bit_xor : Std::U8 -> Std::U8 -> Std::U8`](#bit_xor--stdu8---stdu8---stdu8)
    - [`maximum : U8`](#maximum--u8)
    - [`minimum : U8`](#minimum--u8)
    - [`shift_left : Std::U8 -> Std::U8 -> Std::U8`](#shift_left--stdu8---stdu8---stdu8)
    - [`shift_right : Std::U8 -> Std::U8 -> Std::U8`](#shift_right--stdu8---stdu8---stdu8)
    - [`to_CChar : Std::U8 -> Std::I8`](#to_cchar--stdu8---stdi8)
    - [`to_CDouble : Std::U8 -> Std::F64`](#to_cdouble--stdu8---stdf64)
    - [`to_CFloat : Std::U8 -> Std::F32`](#to_cfloat--stdu8---stdf32)
    - [`to_CInt : Std::U8 -> Std::I32`](#to_cint--stdu8---stdi32)
    - [`to_CLong : Std::U8 -> Std::I64`](#to_clong--stdu8---stdi64)
    - [`to_CLongLong : Std::U8 -> Std::I64`](#to_clonglong--stdu8---stdi64)
    - [`to_CShort : Std::U8 -> Std::I16`](#to_cshort--stdu8---stdi16)
    - [`to_CSizeT : Std::U8 -> Std::U64`](#to_csizet--stdu8---stdu64)
    - [`to_CUnsignedChar : Std::U8 -> Std::U8`](#to_cunsignedchar--stdu8---stdu8)
    - [`to_CUnsignedInt : Std::U8 -> Std::U32`](#to_cunsignedint--stdu8---stdu32)
    - [`to_CUnsignedLong : Std::U8 -> Std::U64`](#to_cunsignedlong--stdu8---stdu64)
    - [`to_CUnsignedLongLong : Std::U8 -> Std::U64`](#to_cunsignedlonglong--stdu8---stdu64)
    - [`to_CUnsignedShort : Std::U8 -> Std::U16`](#to_cunsignedshort--stdu8---stdu16)
    - [`to_F32 : Std::U8 -> Std::F32`](#to_f32--stdu8---stdf32)
    - [`to_F64 : Std::U8 -> Std::F64`](#to_f64--stdu8---stdf64)
    - [`to_I16 : Std::U8 -> Std::I16`](#to_i16--stdu8---stdi16)
    - [`to_I32 : Std::U8 -> Std::I32`](#to_i32--stdu8---stdi32)
    - [`to_I64 : Std::U8 -> Std::I64`](#to_i64--stdu8---stdi64)
    - [`to_I8 : Std::U8 -> Std::I8`](#to_i8--stdu8---stdi8)
    - [`to_U16 : Std::U8 -> Std::U16`](#to_u16--stdu8---stdu16)
    - [`to_U32 : Std::U8 -> Std::U32`](#to_u32--stdu8---stdu32)
    - [`to_U64 : Std::U8 -> Std::U64`](#to_u64--stdu8---stdu64)
    - [`to_U8 : Std::U8 -> Std::U8`](#to_u8--stdu8---stdu8)


# Types and aliases

## `namespace Std`

### `type Array a = box { primitive }`

### `type Bool = unbox { primitive }`

### `type Boxed a = box struct { ...fields... }`

Boxed wrapper for a type.

```
type Boxed a = box struct { value : a };
```

#### field `value : a`

### `type ErrMsg = String`

A type (alias) for error message.

### `type F32 = unbox { primitive }`

### `type F64 = unbox { primitive }`

### `type I16 = unbox { primitive }`

### `type I32 = unbox { primitive }`

### `type I64 = unbox { primitive }`

### `type I8 = unbox { primitive }`

### `type IO a = unbox struct { ...fields... }`

#### field `_data : () -> a`

### `type Iterator a = unbox struct { ...fields... }`

Iterator (a.k.a lazy list)

#### field `next : () -> Option (a, Iterator a)`

### `type Lazy = () -> a`

The type of lazily generated values.

You can create a lazy value by `|_| (...an expression to generate the value...)`, and
you can evaluate a lazy value `v` by `v()`.

### `type LoopResult s b = unbox union { ...variants... }`

#### variant `continue : s`

#### variant `break : b`

### `type Option a = unbox union { ...variants... }`

#### variant `none : ()`

#### variant `some : a`

### `type Path = unbox struct { ...fields... }`

The type for file path.

TODO: give better implementation.

#### field `_data : String`

### `type Ptr = unbox { primitive }`

### `type PunchedArray a = unbox struct { ...fields... }`

The type of punched arrays.

A punched array is an array from which a certain element has been removed.
This is used in the implementation of `Array::act`.

#### field `_data : Destructor (Array a)`

#### field `idx : I64`

### `type Result e o = unbox union { ...variants... }`

A type of result value for a computation that may fail.

#### variant `ok : o`

#### variant `err : e`

### `type String = unbox struct { ...fields... }`

#### field `_data : Array U8`

### `type U16 = unbox { primitive }`

### `type U32 = unbox { primitive }`

### `type U64 = unbox { primitive }`

### `type U8 = unbox { primitive }`

## `namespace Std::FFI`

### `type CChar = Std::I8`

### `type CDouble = Std::F64`

### `type CFloat = Std::F32`

### `type CInt = Std::I32`

### `type CLong = Std::I64`

### `type CLongLong = Std::I64`

### `type CShort = Std::I16`

### `type CSizeT = Std::U64`

### `type CUnsignedChar = Std::U8`

### `type CUnsignedInt = Std::U32`

### `type CUnsignedLong = Std::U64`

### `type CUnsignedLongLong = Std::U64`

### `type CUnsignedShort = Std::U16`

### `type Destructor a = box struct { ...fields... }`

`Destructor a` is a boxed type which is containing a value of type `a` and a function `a -> ()` which is called destructor.
When a value of `Destructor a` is deallocated, the destructor function will be called on the contained value.

This type is useful to free a resouce allocated by a C function automatically when the resource is no longer needed in Fix code.

NOTE1: Accessing the contained value directly by the field accessor function is not recommended. Use `borrow` function to access the value.

NOTE2: If the contained value is captured by another Fix's object than `Destructor`, the contained value is still alive after the destructor function is called.

#### field `_value : a`

#### field `dtor : a -> ()`

## `namespace Std::IO`

### `type IOFail a = unbox struct { ...fields... }`

The type for I/O actions which may fail.

#### field `_data : IO (Result ErrMsg a)`

### `type IOHandle = unbox struct { ...fields... }`

A handle type for read / write operations on files, stdin, stdout, stderr.

You can create `IOHandle` value by `IO::open_file`, and close it by `IO::close_file`.

There are also global `IO::IOHandle::stdin`, `IO::IOHandle::stdout`, `IO::IOHandle::stderr`.

#### field `_data : Destructor Ptr`

# Traits and aliases

## `namespace Std`

### `trait a : Add`

Trait for infix operator `+`.

#### method `add : a -> a -> a`

Adds two values. An expression `x + y` is translated to `add(x, y)`.

### `trait a : Div`

Trait for infix operator `/`.

#### method `div : a -> a -> a`

Divides a value by another value. An expression `x / y` is translated to `div(x, y)`.

### `trait a : Eq`

Trait for infix operator `==`.

#### method `eq : a -> a -> Bool`

Checks equality of two values. An expression `x == y` is translated to `eq(x, y)`.

### `trait a : FromBytes`

#### method `from_bytes : Array U8 -> Result ErrMsg a`

### `trait a : FromString`

#### method `from_string : String -> Result ErrMsg a`

### `trait [f : *->*] f : Functor`

#### method `map : (a -> b) -> f a -> f b`

### `trait a : LessThan`

Trait for infix operator `<`.

#### method `less_than : a -> a -> Bool`

Compares two values. An expression `x < y` is translated to `less_than(x, y)`.

### `trait a : LessThanOrEq`

Trait for infix operator `<=`.

#### method `less_than_or_eq : a -> a -> Bool`

Compares two values. An expression `x <= y` is translated to `less_than_or_eq(x, y)`.

### `trait [m : *->*] m : Monad`

#### method `bind : (a -> m b) -> m a -> m b`

#### method `pure : a -> m a`

### `trait a : Mul`

Trait for infix operator `*`.

#### method `mul : a -> a -> a`

Multiplies a value by another value. An expression `x * y` is translated to `mul(x, y)`.

### `trait a : Neg`

Trait for prefix operator `-`.

#### method `neg : a -> a`

Negates a value. An expression `-x` is translated to `neg(x)`.

### `trait a : Not`

Trait for prefix operator `!`.

#### method `not : a -> a`

Logical NOT of a value. An expression `!x` is translated to `not(x)`.

### `trait a : Rem`

Trait for infix operator `%`.

#### method `rem : a -> a -> a`

Calculate remainder of a value dividing another value. An expression `x % y` is translated to `rem(x, y)`.

### `trait a : Sub`

Trait for infix operator `-`.

#### method `sub : a -> a -> a`

Subtracts a value from another value. An expression `x - y` is translated to `sub(x, y)`.

### `trait a : ToBytes`

#### method `to_bytes : a -> Array U8`

### `trait a : ToString`

#### method `to_string : a -> String`

### `trait a : Zero`

#### method `zero : a`

# Trait implementations

### `impl () : Eq`

### `impl () : ToString`

Returns "()".

### `impl Array : Functor`

### `impl Array : Monad`

### `impl [a : Eq] Array a : Eq`

### `impl [a : Eq, a : LessThan] Array a : LessThan`

`LessThan` implementation for `Array a`.

Compares two arrays by lexicographic order.

### `impl [a : Eq, a : LessThanOrEq] Array a : LessThanOrEq`

`LessThanOrEq` implementation for `Array a`.

Compares two arrays by lexicographic order.

### `impl [a : ToString] Array a : ToString`

### `impl Bool : ToString`

### `impl F32 : FromBytes`

### `impl F32 : FromString`

### `impl F32 : ToBytes`

### `impl F32 : ToString`

### `impl F32 : Zero`

### `impl F64 : FromBytes`

### `impl F64 : FromString`

### `impl F64 : ToBytes`

### `impl F64 : ToString`

### `impl F64 : Zero`

### `impl I16 : FromBytes`

### `impl I16 : FromString`

### `impl I16 : ToBytes`

### `impl I16 : ToString`

### `impl I16 : Zero`

### `impl I32 : FromBytes`

### `impl I32 : FromString`

### `impl I32 : ToBytes`

### `impl I32 : ToString`

### `impl I32 : Zero`

### `impl I64 : FromBytes`

### `impl I64 : FromString`

### `impl I64 : ToBytes`

### `impl I64 : ToString`

### `impl I64 : Zero`

### `impl I8 : FromBytes`

### `impl I8 : FromString`

### `impl I8 : ToBytes`

### `impl I8 : ToString`

### `impl I8 : Zero`

### `impl IO : Functor`

### `impl IO : Monad`

### `impl IOFail : Functor`

### `impl IOFail : Monad`

### `impl Iterator : Functor`

### `impl Iterator : Monad`

### `impl Iterator a : Add`

### `impl [a : Eq] Iterator a : Eq`

### `impl Option : Functor`

### `impl Option : Monad`

### `impl [a : Eq] Option a : Eq`

### `impl [a : ToString] Option a : ToString`

### `impl Path : ToString`

### `impl Ptr : ToString`

### `impl Result e : Functor`

### `impl Result e : Monad`

### `impl [e : Eq, a : Eq] Result e a : Eq`

### `impl [e : ToString, a : ToString] Result e a : ToString`

### `impl Std::Bool : Std::Eq`

### `impl Std::Bool : Std::Not`

### `impl Std::F32 : Std::Add`

### `impl Std::F32 : Std::Div`

### `impl Std::F32 : Std::Eq`

### `impl Std::F32 : Std::LessThan`

### `impl Std::F32 : Std::LessThanOrEq`

### `impl Std::F32 : Std::Mul`

### `impl Std::F32 : Std::Neg`

### `impl Std::F32 : Std::Sub`

### `impl Std::F64 : Std::Add`

### `impl Std::F64 : Std::Div`

### `impl Std::F64 : Std::Eq`

### `impl Std::F64 : Std::LessThan`

### `impl Std::F64 : Std::LessThanOrEq`

### `impl Std::F64 : Std::Mul`

### `impl Std::F64 : Std::Neg`

### `impl Std::F64 : Std::Sub`

### `impl Std::I16 : Std::Add`

### `impl Std::I16 : Std::Div`

### `impl Std::I16 : Std::Eq`

### `impl Std::I16 : Std::LessThan`

### `impl Std::I16 : Std::LessThanOrEq`

### `impl Std::I16 : Std::Mul`

### `impl Std::I16 : Std::Neg`

### `impl Std::I16 : Std::Rem`

### `impl Std::I16 : Std::Sub`

### `impl Std::I32 : Std::Add`

### `impl Std::I32 : Std::Div`

### `impl Std::I32 : Std::Eq`

### `impl Std::I32 : Std::LessThan`

### `impl Std::I32 : Std::LessThanOrEq`

### `impl Std::I32 : Std::Mul`

### `impl Std::I32 : Std::Neg`

### `impl Std::I32 : Std::Rem`

### `impl Std::I32 : Std::Sub`

### `impl Std::I64 : Std::Add`

### `impl Std::I64 : Std::Div`

### `impl Std::I64 : Std::Eq`

### `impl Std::I64 : Std::LessThan`

### `impl Std::I64 : Std::LessThanOrEq`

### `impl Std::I64 : Std::Mul`

### `impl Std::I64 : Std::Neg`

### `impl Std::I64 : Std::Rem`

### `impl Std::I64 : Std::Sub`

### `impl Std::I8 : Std::Add`

### `impl Std::I8 : Std::Div`

### `impl Std::I8 : Std::Eq`

### `impl Std::I8 : Std::LessThan`

### `impl Std::I8 : Std::LessThanOrEq`

### `impl Std::I8 : Std::Mul`

### `impl Std::I8 : Std::Neg`

### `impl Std::I8 : Std::Rem`

### `impl Std::I8 : Std::Sub`

### `impl Std::Ptr : Std::Eq`

### `impl Std::U16 : Std::Add`

### `impl Std::U16 : Std::Div`

### `impl Std::U16 : Std::Eq`

### `impl Std::U16 : Std::LessThan`

### `impl Std::U16 : Std::LessThanOrEq`

### `impl Std::U16 : Std::Mul`

### `impl Std::U16 : Std::Neg`

### `impl Std::U16 : Std::Rem`

### `impl Std::U16 : Std::Sub`

### `impl Std::U32 : Std::Add`

### `impl Std::U32 : Std::Div`

### `impl Std::U32 : Std::Eq`

### `impl Std::U32 : Std::LessThan`

### `impl Std::U32 : Std::LessThanOrEq`

### `impl Std::U32 : Std::Mul`

### `impl Std::U32 : Std::Neg`

### `impl Std::U32 : Std::Rem`

### `impl Std::U32 : Std::Sub`

### `impl Std::U64 : Std::Add`

### `impl Std::U64 : Std::Div`

### `impl Std::U64 : Std::Eq`

### `impl Std::U64 : Std::LessThan`

### `impl Std::U64 : Std::LessThanOrEq`

### `impl Std::U64 : Std::Mul`

### `impl Std::U64 : Std::Neg`

### `impl Std::U64 : Std::Rem`

### `impl Std::U64 : Std::Sub`

### `impl Std::U8 : Std::Add`

### `impl Std::U8 : Std::Div`

### `impl Std::U8 : Std::Eq`

### `impl Std::U8 : Std::LessThan`

### `impl Std::U8 : Std::LessThanOrEq`

### `impl Std::U8 : Std::Mul`

### `impl Std::U8 : Std::Neg`

### `impl Std::U8 : Std::Rem`

### `impl Std::U8 : Std::Sub`

### `impl String : Add`

Concatenates two strings.

### `impl String : Eq`

### `impl String : LessThan`

### `impl String : LessThanOrEq`

### `impl String : ToString`

### `impl U16 : FromBytes`

### `impl U16 : FromString`

### `impl U16 : ToBytes`

### `impl U16 : ToString`

### `impl U16 : Zero`

### `impl U32 : FromBytes`

### `impl U32 : FromString`

### `impl U32 : ToBytes`

### `impl U32 : ToString`

### `impl U32 : Zero`

### `impl U64 : FromBytes`

### `impl U64 : FromString`

### `impl U64 : ToBytes`

### `impl U64 : ToString`

### `impl U64 : Zero`

### `impl U8 : FromBytes`

### `impl U8 : FromString`

### `impl U8 : ToBytes`

### `impl U8 : ToString`

### `impl U8 : Zero`

# Values

## `namespace Std`

### `compose : (a -> b) -> (b -> c) -> a -> c`

Composes two functions. Composition operators `<<` and `>>` is translated to use of `compose`.

### `fix : ((a -> b) -> a -> b) -> a -> b`

`fix` enables you to make a recursive function locally.

The idiom is `fix $ |loop, arg| -> {loop_body}`. In `{loop_body}`, you can call `loop` to make a recursion.

Example:
```
module Main;

main : IO ();
main = (
    let fact = fix $ |loop, n| if n == 0 { 1 } else { n * loop (n-1) };
    println $ fact(5).to_string // evaluates to 5 * 4 * 3 * 2 * 1 = 120
);
```

### `loop : s -> (s -> Std::LoopResult s b) -> b`

`loop` enables you to make a loop. `LoopResult` is a union type defined as follows: 

```
type LoopResult s r = unbox union { continue : s, break : r };
```

`loop` takes two arguments: the initial state of the loop `s0` and the loop body function `body`. 
It first calls `body` on `s0`. 
If `body` returns `break(r)`, then the loop ends and returns `r` as the result. 
If `body` returns `continue(s)`, then the loop calls again `body` on `s`.

Example:
```
module Main;
    
main : IO ();
main = (
    let sum = loop((0, 0), |(i, sum)|
        if i == 100 { break $ sum };
        continue $ (i + 1, sum + i)
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

### `mark_threaded : a -> a`

Traverses all values reachable from the given value, and changes the reference counters of them into multi-threaded mode.

### `undefined : Std::Lazy a`

An undefined value.

Since `undefined()` has generic type `a`, you can put it anywhere and it will be type-checked.
This is useful as a placeholder value that you haven't implemented yet.

Calling this value aborts the execution of the program (calls `abort` in libc).

### `unsafe_is_unique : a -> (Std::Bool, a)`

This function checks if a value is uniquely referenced by a name, and returns the result paired with the given value itself. An unboxed value is always considered unique.

NOTE: Changing outputs of your function depending on uniqueness breaks the referential transparency of the function. If you want to assert that a value is unique, consider using `Debug::assert_unique` instead.

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

## `namespace Std::Array`

### `@ : Std::I64 -> Std::Array a -> a`

Gets an element of an array at the specified index.

### `_get_ptr : Std::Array a -> Std::Ptr`

Get the pointer to the memory region where elements are stored.

This function is dangerous because if the array is not used after call of this function, the array will be deallocated soon and the returned pointer will be dangling.
Try using `borrow_ptr` instead.

### `_get_sub_size_asif : I64 -> I64 -> I64 -> I64 -> Array a -> Array a`

A function like `get_sub`, but behaves as if the size of the array is the specified value,

and has a parameter to specify additional capacity of the returned `Array`.

### `_sort_range_using_buffer : Array a -> I64 -> I64 -> ((a, a) -> Bool) -> Array a -> (Array a, Array a)`

Sorts elements in a range of a vector by "less than" comparator.

This function receives a working buffer as the first argument to reduce memory allocation, and returns it as second element.

### `_unsafe_get : Std::I64 -> Std::Array a -> a`

Gets a value from an array, without bounds checking and retaining the returned value.

### `_unsafe_set : Std::I64 -> a -> Std::Array a -> Std::Array a`

Sets a value into an array, without uniqueness checking, bounds checking and releasing the old value.

### `_unsafe_set_size : Std::I64 -> Std::Array a -> Std::Array a`

Updates the length of an array, without uniqueness checking or validation of the given length value.

### `act : [f : Functor] I64 -> (a -> f a) -> Array a -> f (Array a)`

Modifies an array by a functorial action.

Semantically, `arr.act(idx, fun)` is equivalent to `fun(arr.@(idx)).map(|elm| arr.set(idx, elm))`.

This function can be defined for any functor `f` in general, but it is easier to understand the behavior when `f` is a monad:
the monadic action `act(idx, fun, arr)` first performs `fun(arr.@(idx))` to get a value `elm`, and returns a pure value `arr.set(idx, elm)`.

If you call `arr.act(idx, fun)` when both of `arr` and `arr.@(idx)` are unique, it is assured that `fun` receives the unique value.

If you call `act` on an array which is shared, this function clones the given array when inserting the result of your action into the array.
This means that you don't need to pay cloning cost when your action failed, as expected.

### `append : Array a -> Array a -> Array a`

Appends an array to an array.

Note: Since `a1.append(a2)` puts `a2` after `a1`, `append(lhs, rhs)` puts `lhs` after `rhs`.

### `borrow_ptr : (Ptr -> b) -> Array a -> b`

Calls a function with a pointer to the memory region where elements are stored.

### `empty : Std::I64 -> Std::Array a`

Creates an empty array with specified capacity.

### `fill : Std::I64 -> a -> Std::Array a`

Creates an array of the specified length filled with the initial value.

The capacity is set to the same value as the length.

Example: `fill(n, x) == [x, x, x, ..., x]` (of length `n`).

### `find_by : (a -> Bool) -> Array a -> Option I64`

Finds the first index at which the element satisfies a condition.

### `force_unique : Std::Array a -> Std::Array a`

Force the uniqueness of an array.
If the given array is shared, this function returns the cloned array.

### `from_iter : Iterator a -> Array a`

Create an array from an iterator.

### `from_map : I64 -> (I64 -> a) -> Array a`

Creates an array by a mapping function.

### `get_capacity : Std::Array a -> Std::I64`

Gets the capacity of an array.

### `get_first : Array a -> Option a`

Gets the first element of an array. Returns none if the array is empty.

### `get_last : Array a -> Option a`

Gets the last element of an array. Returns none if the array is empty.

### `get_size : Std::Array a -> Std::I64`

Gets the length of an array.

### `get_sub : I64 -> I64 -> Array a -> Array a`

`arr.get_sub(s, e)` returns an array `[ arr.@(i) | i ∈ [s, e) ]`,

More precisely, let `N` denote the the size of the `arr`.
Then `arr.get_sub(s, e)` returns `[ arr.@(s + i mod N) | i ∈ [0, n), n >= 0 is the minimum number such that s + n == e mod N ]`.

### `is_empty : Array a -> Bool`

Returns if the array is empty

### `mod : Std::I64 -> (a -> a) -> Std::Array a -> Std::Array a`

Updates an array by applying a function to the element at the specified index.

This function clones the given array if it is shared.

If you call `arr.mod(i, f)` when both of `arr` and `arr.@(i)` are unique, it is assured that `f` receives the element value which is unique.

### `pop_back : Array a -> Array a`

Pops an element at the back of an array.
If the array is empty, this function does nothing.

### `push_back : a -> Array a -> Array a`

Pushes an element to the back of an array.

### `reserve : I64 -> Array a -> Array a`

Reserves the memory region for an array.

TODO: change to more optimized implementation.

### `set : Std::I64 -> a -> Std::Array a -> Std::Array a`

Updates an array by setting a value as the element at the specified index.

This function clones the given array if it is shared.

### `sort_by : ((a, a) -> Bool) -> Array a -> Array a`

Sorts elements in a vector by "less than" comparator.

### `to_iter : Array a -> Iterator a`

Converts an array to an iterator.

### `truncate : I64 -> Array a -> Array a`

Truncates an array, keeping the given number of first elements.

`truncante(len, arr)` does nothing if `len >= arr.get_size`.

## `namespace Std::F32`

### `abs : F32 -> F32`

### `infinity : Std::F32`

The infinity value for the given floating point type.

### `quiet_nan : Std::F32`

A floating number represented by `01...1` in binary.

### `to_CChar : Std::F32 -> Std::I8`

Casts a value of `F32` into a value of `CChar`.

### `to_CDouble : Std::F32 -> Std::F64`

Casts a value of `F32` into a value of `CDouble`.

### `to_CFloat : Std::F32 -> Std::F32`

Casts a value of `F32` into a value of `CFloat`.

### `to_CInt : Std::F32 -> Std::I32`

Casts a value of `F32` into a value of `CInt`.

### `to_CLong : Std::F32 -> Std::I64`

Casts a value of `F32` into a value of `CLong`.

### `to_CLongLong : Std::F32 -> Std::I64`

Casts a value of `F32` into a value of `CLongLong`.

### `to_CShort : Std::F32 -> Std::I16`

Casts a value of `F32` into a value of `CShort`.

### `to_CSizeT : Std::F32 -> Std::U64`

Casts a value of `F32` into a value of `CSizeT`.

### `to_CUnsignedChar : Std::F32 -> Std::U8`

Casts a value of `F32` into a value of `CUnsignedChar`.

### `to_CUnsignedInt : Std::F32 -> Std::U32`

Casts a value of `F32` into a value of `CUnsignedInt`.

### `to_CUnsignedLong : Std::F32 -> Std::U64`

Casts a value of `F32` into a value of `CUnsignedLong`.

### `to_CUnsignedLongLong : Std::F32 -> Std::U64`

Casts a value of `F32` into a value of `CUnsignedLongLong`.

### `to_CUnsignedShort : Std::F32 -> Std::U16`

Casts a value of `F32` into a value of `CUnsignedShort`.

### `to_F32 : Std::F32 -> Std::F32`

Casts a value of `F32` into a value of `F32`.

### `to_F64 : Std::F32 -> Std::F64`

Casts a value of `F32` into a value of `F64`.

### `to_I16 : Std::F32 -> Std::I16`

Casts a value of `F32` into a value of `I16`.

### `to_I32 : Std::F32 -> Std::I32`

Casts a value of `F32` into a value of `I32`.

### `to_I64 : Std::F32 -> Std::I64`

Casts a value of `F32` into a value of `I64`.

### `to_I8 : Std::F32 -> Std::I8`

Casts a value of `F32` into a value of `I8`.

### `to_U16 : Std::F32 -> Std::U16`

Casts a value of `F32` into a value of `U16`.

### `to_U32 : Std::F32 -> Std::U32`

Casts a value of `F32` into a value of `U32`.

### `to_U64 : Std::F32 -> Std::U64`

Casts a value of `F32` into a value of `U64`.

### `to_U8 : Std::F32 -> Std::U8`

Casts a value of `F32` into a value of `U8`.

### `to_string_exp : F32 -> String`

Converts a floating number to a string of exponential form.

### `to_string_exp_precision : U8 -> F32 -> String`

Converts a floating number to a string of exponential form with specified precision (i.e., number of digits after the decimal point).

### `to_string_precision : U8 -> F32 -> String`

Converts a floating number to a string with specified precision (i.e., number of digits after the decimal point).

## `namespace Std::F64`

### `abs : F64 -> F64`

### `infinity : Std::F64`

The infinity value for the given floating point type.

### `quiet_nan : Std::F64`

A floating number represented by `01...1` in binary.

### `to_CChar : Std::F64 -> Std::I8`

Casts a value of `F64` into a value of `CChar`.

### `to_CDouble : Std::F64 -> Std::F64`

Casts a value of `F64` into a value of `CDouble`.

### `to_CFloat : Std::F64 -> Std::F32`

Casts a value of `F64` into a value of `CFloat`.

### `to_CInt : Std::F64 -> Std::I32`

Casts a value of `F64` into a value of `CInt`.

### `to_CLong : Std::F64 -> Std::I64`

Casts a value of `F64` into a value of `CLong`.

### `to_CLongLong : Std::F64 -> Std::I64`

Casts a value of `F64` into a value of `CLongLong`.

### `to_CShort : Std::F64 -> Std::I16`

Casts a value of `F64` into a value of `CShort`.

### `to_CSizeT : Std::F64 -> Std::U64`

Casts a value of `F64` into a value of `CSizeT`.

### `to_CUnsignedChar : Std::F64 -> Std::U8`

Casts a value of `F64` into a value of `CUnsignedChar`.

### `to_CUnsignedInt : Std::F64 -> Std::U32`

Casts a value of `F64` into a value of `CUnsignedInt`.

### `to_CUnsignedLong : Std::F64 -> Std::U64`

Casts a value of `F64` into a value of `CUnsignedLong`.

### `to_CUnsignedLongLong : Std::F64 -> Std::U64`

Casts a value of `F64` into a value of `CUnsignedLongLong`.

### `to_CUnsignedShort : Std::F64 -> Std::U16`

Casts a value of `F64` into a value of `CUnsignedShort`.

### `to_F32 : Std::F64 -> Std::F32`

Casts a value of `F64` into a value of `F32`.

### `to_F64 : Std::F64 -> Std::F64`

Casts a value of `F64` into a value of `F64`.

### `to_I16 : Std::F64 -> Std::I16`

Casts a value of `F64` into a value of `I16`.

### `to_I32 : Std::F64 -> Std::I32`

Casts a value of `F64` into a value of `I32`.

### `to_I64 : Std::F64 -> Std::I64`

Casts a value of `F64` into a value of `I64`.

### `to_I8 : Std::F64 -> Std::I8`

Casts a value of `F64` into a value of `I8`.

### `to_U16 : Std::F64 -> Std::U16`

Casts a value of `F64` into a value of `U16`.

### `to_U32 : Std::F64 -> Std::U32`

Casts a value of `F64` into a value of `U32`.

### `to_U64 : Std::F64 -> Std::U64`

Casts a value of `F64` into a value of `U64`.

### `to_U8 : Std::F64 -> Std::U8`

Casts a value of `F64` into a value of `U8`.

### `to_string_exp : F64 -> String`

Converts a floating number to a string of exponential form.

### `to_string_exp_precision : U8 -> F64 -> String`

Converts a floating number to a string of exponential form with specified precision (i.e., number of digits after the decimal point).

### `to_string_precision : U8 -> F64 -> String`

Converts a floating number to a string with specified precision (i.e., number of digits after the decimal point).

## `namespace Std::FFI`

### `_unsafe_get_boxed_data_ptr : a -> Std::Ptr`

Returns a pointer to the data of a boxed value.

The difference from `unsafe_get_retained_ptr_of_boxed_value` is that this function returns a pointer to region where the payload of a boxed value is stored;
on the other hand, `unsafe_get_retained_ptr_of_boxed_value` returns a pointer to the boxed value itself (i.e., the control block of the value).

Note that if the call `v._unsafe_get_boxed_data_ptr` is the last usage of `v`, then this function deallocates `v` and returns a dangling pointer.
To avoid issues caused by this, use `unsafe_borrow_boxed_data_ptr` instead.

### `unsafe_borrow_boxed_data_ptr : (Ptr -> b) -> a -> b`

Borrows a pointer to the data of a boxed value.

For more details, see the document of `_unsafe_get_boxed_data_ptr`.

### `unsafe_clear_errno : () -> ()`

Sets errno to zero.

### `unsafe_get_boxed_value_from_retained_ptr : Std::Ptr -> a`

Creates a boxed value from a retained pointer obtained by `unsafe_get_retained_ptr_of_boxed_value`.

### `unsafe_get_errno : () -> CInt`

Gets errno which is set by C functions.

### `unsafe_get_release_function_of_boxed_value : Std::Lazy a -> Std::Ptr`

Returns a pointer to the function of type `void (*)(void*)` which releases a boxed value of type `a`.
This function is used to release a pointer obtained by `_unsafe_get_retained_ptr_of_boxed_value`.

Note that this function is requires a value of type `Lazy a`, not of `a`.
So you can get release function for a boxed type `T` even when you don't have a value of type `T` -- you can just use `|_| undefined() : T`:

```
module Main;

type VoidType = box struct {};
// No constructor for `VoidType` is provided.

main: IO ();
main = (
    let release = (|_| undefined() : VoidType).unsafe_get_release_function_of_boxed_value; // Release function of `VoidType`.
    pure()
);
```

In case the type is not a specific `T`, but a generic parameter `a` that appears in the type signature of a function you are implementing, you cannot use the above technique, because writing `|_| undefined() : a` is not allowed in Fix's syntax. Even in such a case, if you have some value related to `a`, you can make a `Lazy a` value in many cases. For example:
- If you have a function `f : b -> a`, then you can use `|_| f(undefined())` of type `Lazy a`. 
- If you have a function `f : a -> b`, then you can use `|_| let x = undefined(); let _ = f(x); x` of type `Lazy a`.

### `unsafe_get_retain_function_of_boxed_value : Std::Lazy a -> Std::Ptr`

Returns a pointer to the function of type `void (*)(void*)` which retains a boxed value of type `a`.
This function is used to retain a pointer obtained by `_unsafe_get_retained_ptr_of_boxed_value`.

For the reason that this function requires a value of type `Lazy a`, not of `a`, see the document for `unsafe_get_release_function_of_boxed_value`.

### `unsafe_get_retained_ptr_of_boxed_value : a -> Std::Ptr`

Returns a retained pointer to a boxed value.
This function is used to share ownership of Fix's boxed values with foreign languages.

To get back the boxed value from the retained pointer, use `unsafe_get_boxed_value_from_retained_ptr`.
To release / retain the value in a foreign language, call the function pointer obtained by `unsafe_get_release_function_of_boxed_value` or `unsafe_get_retain_function_of_boxed_value` on the pointer.

Note that the returned pointer points to the control block allocated by Fix, and does not necessary points to the data of the boxed value.
If you want to get a pointer to the data of the boxed value, use `unsafe_borrow_boxed_data_ptr`.

## `namespace Std::FFI::Destructor`

### `borrow : (a -> b) -> Destructor a -> b`

Borrow the contained value.
`borrow(worker, dtor)` calls `worker` on the contained value captured by `dtor`, and returns the value returned by `worker`.
It is guaranteed that the `dtor` is alive during the call of `worker`.
In other words, the `worker` receives the contained value on which the destructor is not called yet.

### `make : a -> (a -> ()) -> Destructor a`

Make a destructor value.

## `namespace Std::Functor`

### `forget : [f : Functor] f a -> f ()`

## `namespace Std::I16`

### `abs : I16 -> I16`

### `bit_and : Std::I16 -> Std::I16 -> Std::I16`

Calculates bitwise AND of two values.

### `bit_or : Std::I16 -> Std::I16 -> Std::I16`

Calculates bitwise OR of two values.

### `bit_xor : Std::I16 -> Std::I16 -> Std::I16`

Calculates bitwise XOR of two values.

### `maximum : I16`

### `minimum : I16`

### `shift_left : Std::I16 -> Std::I16 -> Std::I16`

`v.shift_left(w)` shifts `v` to left by `w` bits.

### `shift_right : Std::I16 -> Std::I16 -> Std::I16`

`v.shift_right(w)` shifts `v` to right by `w` bits.

### `to_CChar : Std::I16 -> Std::I8`

Casts a value of `I16` into a value of `CChar`.

### `to_CDouble : Std::I16 -> Std::F64`

Casts a value of `I16` into a value of `CDouble`.

### `to_CFloat : Std::I16 -> Std::F32`

Casts a value of `I16` into a value of `CFloat`.

### `to_CInt : Std::I16 -> Std::I32`

Casts a value of `I16` into a value of `CInt`.

### `to_CLong : Std::I16 -> Std::I64`

Casts a value of `I16` into a value of `CLong`.

### `to_CLongLong : Std::I16 -> Std::I64`

Casts a value of `I16` into a value of `CLongLong`.

### `to_CShort : Std::I16 -> Std::I16`

Casts a value of `I16` into a value of `CShort`.

### `to_CSizeT : Std::I16 -> Std::U64`

Casts a value of `I16` into a value of `CSizeT`.

### `to_CUnsignedChar : Std::I16 -> Std::U8`

Casts a value of `I16` into a value of `CUnsignedChar`.

### `to_CUnsignedInt : Std::I16 -> Std::U32`

Casts a value of `I16` into a value of `CUnsignedInt`.

### `to_CUnsignedLong : Std::I16 -> Std::U64`

Casts a value of `I16` into a value of `CUnsignedLong`.

### `to_CUnsignedLongLong : Std::I16 -> Std::U64`

Casts a value of `I16` into a value of `CUnsignedLongLong`.

### `to_CUnsignedShort : Std::I16 -> Std::U16`

Casts a value of `I16` into a value of `CUnsignedShort`.

### `to_F32 : Std::I16 -> Std::F32`

Casts a value of `I16` into a value of `F32`.

### `to_F64 : Std::I16 -> Std::F64`

Casts a value of `I16` into a value of `F64`.

### `to_I16 : Std::I16 -> Std::I16`

Casts a value of `I16` into a value of `I16`.

### `to_I32 : Std::I16 -> Std::I32`

Casts a value of `I16` into a value of `I32`.

### `to_I64 : Std::I16 -> Std::I64`

Casts a value of `I16` into a value of `I64`.

### `to_I8 : Std::I16 -> Std::I8`

Casts a value of `I16` into a value of `I8`.

### `to_U16 : Std::I16 -> Std::U16`

Casts a value of `I16` into a value of `U16`.

### `to_U32 : Std::I16 -> Std::U32`

Casts a value of `I16` into a value of `U32`.

### `to_U64 : Std::I16 -> Std::U64`

Casts a value of `I16` into a value of `U64`.

### `to_U8 : Std::I16 -> Std::U8`

Casts a value of `I16` into a value of `U8`.

## `namespace Std::I32`

### `abs : I32 -> I32`

### `bit_and : Std::I32 -> Std::I32 -> Std::I32`

Calculates bitwise AND of two values.

### `bit_or : Std::I32 -> Std::I32 -> Std::I32`

Calculates bitwise OR of two values.

### `bit_xor : Std::I32 -> Std::I32 -> Std::I32`

Calculates bitwise XOR of two values.

### `maximum : I32`

### `minimum : I32`

### `shift_left : Std::I32 -> Std::I32 -> Std::I32`

`v.shift_left(w)` shifts `v` to left by `w` bits.

### `shift_right : Std::I32 -> Std::I32 -> Std::I32`

`v.shift_right(w)` shifts `v` to right by `w` bits.

### `to_CChar : Std::I32 -> Std::I8`

Casts a value of `I32` into a value of `CChar`.

### `to_CDouble : Std::I32 -> Std::F64`

Casts a value of `I32` into a value of `CDouble`.

### `to_CFloat : Std::I32 -> Std::F32`

Casts a value of `I32` into a value of `CFloat`.

### `to_CInt : Std::I32 -> Std::I32`

Casts a value of `I32` into a value of `CInt`.

### `to_CLong : Std::I32 -> Std::I64`

Casts a value of `I32` into a value of `CLong`.

### `to_CLongLong : Std::I32 -> Std::I64`

Casts a value of `I32` into a value of `CLongLong`.

### `to_CShort : Std::I32 -> Std::I16`

Casts a value of `I32` into a value of `CShort`.

### `to_CSizeT : Std::I32 -> Std::U64`

Casts a value of `I32` into a value of `CSizeT`.

### `to_CUnsignedChar : Std::I32 -> Std::U8`

Casts a value of `I32` into a value of `CUnsignedChar`.

### `to_CUnsignedInt : Std::I32 -> Std::U32`

Casts a value of `I32` into a value of `CUnsignedInt`.

### `to_CUnsignedLong : Std::I32 -> Std::U64`

Casts a value of `I32` into a value of `CUnsignedLong`.

### `to_CUnsignedLongLong : Std::I32 -> Std::U64`

Casts a value of `I32` into a value of `CUnsignedLongLong`.

### `to_CUnsignedShort : Std::I32 -> Std::U16`

Casts a value of `I32` into a value of `CUnsignedShort`.

### `to_F32 : Std::I32 -> Std::F32`

Casts a value of `I32` into a value of `F32`.

### `to_F64 : Std::I32 -> Std::F64`

Casts a value of `I32` into a value of `F64`.

### `to_I16 : Std::I32 -> Std::I16`

Casts a value of `I32` into a value of `I16`.

### `to_I32 : Std::I32 -> Std::I32`

Casts a value of `I32` into a value of `I32`.

### `to_I64 : Std::I32 -> Std::I64`

Casts a value of `I32` into a value of `I64`.

### `to_I8 : Std::I32 -> Std::I8`

Casts a value of `I32` into a value of `I8`.

### `to_U16 : Std::I32 -> Std::U16`

Casts a value of `I32` into a value of `U16`.

### `to_U32 : Std::I32 -> Std::U32`

Casts a value of `I32` into a value of `U32`.

### `to_U64 : Std::I32 -> Std::U64`

Casts a value of `I32` into a value of `U64`.

### `to_U8 : Std::I32 -> Std::U8`

Casts a value of `I32` into a value of `U8`.

## `namespace Std::I64`

### `abs : I64 -> I64`

### `bit_and : Std::I64 -> Std::I64 -> Std::I64`

Calculates bitwise AND of two values.

### `bit_or : Std::I64 -> Std::I64 -> Std::I64`

Calculates bitwise OR of two values.

### `bit_xor : Std::I64 -> Std::I64 -> Std::I64`

Calculates bitwise XOR of two values.

### `maximum : I64`

### `minimum : I64`

### `shift_left : Std::I64 -> Std::I64 -> Std::I64`

`v.shift_left(w)` shifts `v` to left by `w` bits.

### `shift_right : Std::I64 -> Std::I64 -> Std::I64`

`v.shift_right(w)` shifts `v` to right by `w` bits.

### `to_CChar : Std::I64 -> Std::I8`

Casts a value of `I64` into a value of `CChar`.

### `to_CDouble : Std::I64 -> Std::F64`

Casts a value of `I64` into a value of `CDouble`.

### `to_CFloat : Std::I64 -> Std::F32`

Casts a value of `I64` into a value of `CFloat`.

### `to_CInt : Std::I64 -> Std::I32`

Casts a value of `I64` into a value of `CInt`.

### `to_CLong : Std::I64 -> Std::I64`

Casts a value of `I64` into a value of `CLong`.

### `to_CLongLong : Std::I64 -> Std::I64`

Casts a value of `I64` into a value of `CLongLong`.

### `to_CShort : Std::I64 -> Std::I16`

Casts a value of `I64` into a value of `CShort`.

### `to_CSizeT : Std::I64 -> Std::U64`

Casts a value of `I64` into a value of `CSizeT`.

### `to_CUnsignedChar : Std::I64 -> Std::U8`

Casts a value of `I64` into a value of `CUnsignedChar`.

### `to_CUnsignedInt : Std::I64 -> Std::U32`

Casts a value of `I64` into a value of `CUnsignedInt`.

### `to_CUnsignedLong : Std::I64 -> Std::U64`

Casts a value of `I64` into a value of `CUnsignedLong`.

### `to_CUnsignedLongLong : Std::I64 -> Std::U64`

Casts a value of `I64` into a value of `CUnsignedLongLong`.

### `to_CUnsignedShort : Std::I64 -> Std::U16`

Casts a value of `I64` into a value of `CUnsignedShort`.

### `to_F32 : Std::I64 -> Std::F32`

Casts a value of `I64` into a value of `F32`.

### `to_F64 : Std::I64 -> Std::F64`

Casts a value of `I64` into a value of `F64`.

### `to_I16 : Std::I64 -> Std::I16`

Casts a value of `I64` into a value of `I16`.

### `to_I32 : Std::I64 -> Std::I32`

Casts a value of `I64` into a value of `I32`.

### `to_I64 : Std::I64 -> Std::I64`

Casts a value of `I64` into a value of `I64`.

### `to_I8 : Std::I64 -> Std::I8`

Casts a value of `I64` into a value of `I8`.

### `to_U16 : Std::I64 -> Std::U16`

Casts a value of `I64` into a value of `U16`.

### `to_U32 : Std::I64 -> Std::U32`

Casts a value of `I64` into a value of `U32`.

### `to_U64 : Std::I64 -> Std::U64`

Casts a value of `I64` into a value of `U64`.

### `to_U8 : Std::I64 -> Std::U8`

Casts a value of `I64` into a value of `U8`.

## `namespace Std::I8`

### `abs : I8 -> I8`

### `bit_and : Std::I8 -> Std::I8 -> Std::I8`

Calculates bitwise AND of two values.

### `bit_or : Std::I8 -> Std::I8 -> Std::I8`

Calculates bitwise OR of two values.

### `bit_xor : Std::I8 -> Std::I8 -> Std::I8`

Calculates bitwise XOR of two values.

### `maximum : I8`

### `minimum : I8`

### `shift_left : Std::I8 -> Std::I8 -> Std::I8`

`v.shift_left(w)` shifts `v` to left by `w` bits.

### `shift_right : Std::I8 -> Std::I8 -> Std::I8`

`v.shift_right(w)` shifts `v` to right by `w` bits.

### `to_CChar : Std::I8 -> Std::I8`

Casts a value of `I8` into a value of `CChar`.

### `to_CDouble : Std::I8 -> Std::F64`

Casts a value of `I8` into a value of `CDouble`.

### `to_CFloat : Std::I8 -> Std::F32`

Casts a value of `I8` into a value of `CFloat`.

### `to_CInt : Std::I8 -> Std::I32`

Casts a value of `I8` into a value of `CInt`.

### `to_CLong : Std::I8 -> Std::I64`

Casts a value of `I8` into a value of `CLong`.

### `to_CLongLong : Std::I8 -> Std::I64`

Casts a value of `I8` into a value of `CLongLong`.

### `to_CShort : Std::I8 -> Std::I16`

Casts a value of `I8` into a value of `CShort`.

### `to_CSizeT : Std::I8 -> Std::U64`

Casts a value of `I8` into a value of `CSizeT`.

### `to_CUnsignedChar : Std::I8 -> Std::U8`

Casts a value of `I8` into a value of `CUnsignedChar`.

### `to_CUnsignedInt : Std::I8 -> Std::U32`

Casts a value of `I8` into a value of `CUnsignedInt`.

### `to_CUnsignedLong : Std::I8 -> Std::U64`

Casts a value of `I8` into a value of `CUnsignedLong`.

### `to_CUnsignedLongLong : Std::I8 -> Std::U64`

Casts a value of `I8` into a value of `CUnsignedLongLong`.

### `to_CUnsignedShort : Std::I8 -> Std::U16`

Casts a value of `I8` into a value of `CUnsignedShort`.

### `to_F32 : Std::I8 -> Std::F32`

Casts a value of `I8` into a value of `F32`.

### `to_F64 : Std::I8 -> Std::F64`

Casts a value of `I8` into a value of `F64`.

### `to_I16 : Std::I8 -> Std::I16`

Casts a value of `I8` into a value of `I16`.

### `to_I32 : Std::I8 -> Std::I32`

Casts a value of `I8` into a value of `I32`.

### `to_I64 : Std::I8 -> Std::I64`

Casts a value of `I8` into a value of `I64`.

### `to_I8 : Std::I8 -> Std::I8`

Casts a value of `I8` into a value of `I8`.

### `to_U16 : Std::I8 -> Std::U16`

Casts a value of `I8` into a value of `U16`.

### `to_U32 : Std::I8 -> Std::U32`

Casts a value of `I8` into a value of `U32`.

### `to_U64 : Std::I8 -> Std::U64`

Casts a value of `I8` into a value of `U64`.

### `to_U8 : Std::I8 -> Std::U8`

Casts a value of `I8` into a value of `U8`.

## `namespace Std::IO`

### `_read_line_inner : Bool -> IOHandle -> IOFail String`

Reads characters from an IOHandle.

If the first argument `upto_newline` is true, this function reads a file upto newline or EOF.

### `_unsafe_perform : IO a -> a`

Performs the I/O action. This may violate purity of Fix.

### `close_file : IOHandle -> IO ()`

Closes a file.

Unlike C's `fclose`, closing an already closed `IOHandle` is safe and does nothing.

### `eprint : String -> IO ()`

Prints a string to stderr.

### `eprintln : String -> IO ()`

Prints a string followed by a newline to stderr.

### `exit : I64 -> IO a`

Exits the program with an error code.

### `exit_with_msg : I64 -> String -> IO a`

Exits the program with an error message and an error code.

The error message is written to the standard error output.

### `from_func : (() -> a) -> IO a`

Creates an IO action from a function.

### `get_arg : I64 -> IO (Option String)`

`get_arg(n)` returns the n-th (0-indexed) command line argument.
If n is greater than or equal to the number of command line arguments, this function returns none.

### `get_arg_count : IO I64`

Gets the number of command line arguments.

### `get_args : IO (Array String)`

Gets command line arguments.

### `input_line : IO String`

Reads a line from stdin. If some error occurr, this function aborts the program.
If you want to handle errors, use `read_line(stdin)` instead.

### `is_eof : IOHandle -> IO Bool`

Checks if an `IOHandle` reached to the EOF.

### `loop_lines : IOHandle -> s -> (s -> String -> LoopResult s s) -> IOFail s`

Loop on lines read from an `IOHandle`.

`loop_lines(handle, initial_state, worker)` calls `worker` on the pair of current state and a line string read from `handle`.
The function `worker` should return an updated state as `LoopResult` value, i.e., a value created by `continue` or `break`.
When the `handle` reaches to the EOF or `worker` returns a `break` value, `loop_lines` returns the last state value.

Note that the line string passed to `worker` may contain a newline code at the end. To remove it, use `String::strip_last_spaces`.

### `loop_lines_io : IOHandle -> s -> (s -> String -> IOFail (LoopResult s s)) -> IOFail s`

Loop on lines read from an `IOHandle`.

Similar to `loop_lines`, but the worker function can perform an IO action.

### `open_file : Path -> String -> IOFail IOHandle`

Openes a file. The second argument is a mode string for `fopen` C function.

### `print : String -> IO ()`

Prints a string to stdout.

### `println : String -> IO ()`

Prints a string followed by a newline to stdout.

### `read_bytes : IOHandle -> IOFail (Array U8)`

Reads all bytes from an IOHandle.

### `read_file_bytes : Path -> IOFail (Array U8)`

Reads all bytes from a file.

### `read_file_string : Path -> IOFail String`

Raads all characters from a file.

### `read_line : IOHandle -> IOFail String`

Reads characters from a IOHandle upto newline or EOF.
The returned string may include newline at its end.

### `read_n_bytes : IOHandle -> I64 -> IOFail (Array U8)`

Reads at most n bytes from an IOHandle.

### `read_string : IOHandle -> IOFail String`

Reads all characters from an IOHandle.

### `stderr : IOHandle`

The handle for standard error.

### `stdin : IOHandle`

The handle for standard input.

### `stdout : IOHandle`

The handle for standard output.

### `with_file : Path -> String -> (IOHandle -> IOFail a) -> IOFail a`

Performs a function with a file handle. The second argument is a mode string for `fopen` C function.

The file handle will be closed automatically.

### `write_bytes : IOHandle -> Array U8 -> IOFail ()`

Writes a byte array into an IOHandle.

### `write_file_bytes : Path -> Array U8 -> IOFail ()`

Writes a byte array into a file.

### `write_file_string : Path -> String -> IOFail ()`

Writes a string into a file.

### `write_string : IOHandle -> String -> IOFail ()`

Writes a string into an IOHandle.

## `namespace Std::IO::IOFail`

### `from_result : Result ErrMsg a -> IOFail a`

Creates an pure `IOFail` value from a `Result` value.

### `lift : IO a -> IOFail a`

Lifts an `IO` action to a successful `IOFail` action.

### `throw : ErrMsg -> IOFail a`

Creates an error `IOFail` action.

### `to_result : IOFail a -> IO (Result ErrMsg a)`

Converts an `IOFail` to an `Result` value (wrapped by `IO`).

### `try : (ErrMsg -> IO a) -> IOFail a -> IO a`

Converts an `IOFail` value to an `IO` value by an error handler (i.e., a `catch`) function.

## `namespace Std::IO::IOHandle`

### `_file_ptr : IOHandle -> Ptr`

Gets pointer to C's `FILE` value from an `IOHandle`.

DO NOT call `fclose` on the pointer returned by this function.
To close an `IOHandle`, use `IO::close_file`.

### `_unsafe_close : IOHandle -> ()`

Closes an `IOHandle`.

This is an I/O action not wrapped by `IO`; use `IO::close_file` in the usual case.

### `from_file_ptr : Ptr -> IOHandle`

Creates an `IOHandle` from a file pointer (i.e., pointer to C's `FILE`).

Creating two `IOHandle`s from a single file pointer is forbidden.

## `namespace Std::Iterator`

### `_flatten : Iterator (Iterator a) -> Iterator a`

Flatten an iterator of iterators.

You should use `Monad::flatten` instead of this function.
This function is used in the implementation of `Monad::bind` for `Iterator`.

### `_flatten_sub : Iterator a -> Iterator (Iterator a) -> Iterator a`

### `advance : Iterator a -> Option (a, Iterator a)`

Gets next value and next iterator.

### `append : Iterator a -> Iterator a -> Iterator a`

Appends an iterator to a iterator.
Note: Since `iter1.append(iter2)` puts `iter2` after `iter1`, `append(lhs, rhs)` puts `lhs` after `rhs`.

### `bang : Iterator a -> Iterator a`

Evaluates all elements of iterator.
TODO: add test

### `count_up : I64 -> Iterator I64`

Creates an iterator that counts up from a number.
count_up(n) = [n, n+1, n+2, ...]

### `empty : Iterator a`

Creates an empty iterator.

### `filter : (a -> Bool) -> Iterator a -> Iterator a`

Filters elements by a condition function

### `find_last : Iterator a -> Option a`

Finds the last element of an iterator.

### `fold : b -> (b -> a -> b) -> Iterator a -> b`

Folds iterator from left to right.
Example: `fold(init, op, [a0, a1, a2, ...]) = ...op(op(op(init, a0), a1), a2)...`

### `fold_m : [m : Monad] b -> (b -> a -> m b) -> Iterator a -> m b`

Folds iterator from left to right by monadic action.

### `from_array : Array a -> Iterator a`

Creates iterator from an array.

### `from_map : (I64 -> a) -> Iterator a`

Creates iterator from mapping function.
from_map(f) = [f(0), f(1), f(2), ...]

### `generate : s -> (s -> Option (a, s)) -> Iterator a`

Generates an iterator from a state transition function.
- if `f(s)` is none, `generate(s, f)` is empty.
- if `f(s)` is some value `(e, s1)`, then `generate(s, f)` starts by `e` followed by `generate(s2, f)`.

### `get_first : Iterator a -> Option a`

Gets the first element of an iterator. If the iterator is empty, this function returns `none`.
TODO: add test

### `get_size : Iterator a -> I64`

Counts the number of elements of an iterator.

### `get_tail : Iterator a -> Option (Iterator a)`

Removes the first element from an iterator. If the iterator is empty, this function returns `none`.
TODO: add test

### `intersperse : a -> Iterator a -> Iterator a`

Intersperse an elemnt between elements of an iterator.

Example:
```
Iterator::from_array([1,2,3]).intersperse(0) == Iterator::from_array([1,0,2,0,3])
```

### `is_empty : Iterator a -> Bool`

Check if the iterator is empty.

### `loop_iter : b -> (b -> a -> LoopResult b b) -> Iterator a -> b`

Loop along an iterator. At each iteration step, you can choose to continue or to break.

### `loop_iter_m : [m : Monad] b -> (b -> a -> m (LoopResult b b)) -> Iterator a -> m b`

Loop by monadic action along an iterator. At each iteration step, you can choose to continue or to break.

### `product : Iterator a -> Iterator b -> Iterator (b, a)`

Generates the cartesian product of two iterators.

Example: `[1, 2, 3].to_iter.product(['a', 'b'].to_iter).to_array == [(1, 'a'), (2, 'a'), (3, 'a'), (1, 'b'), (2, 'b'), (3, 'b')]`

### `push_front : a -> Iterator a -> Iterator a`

Pushes an element to an iterator.

### `range : I64 -> I64 -> Iterator I64`

Creates a range iterator, i.e. an iterator of the form `[a, a+1, a+2, ..., b-1]`.

### `reverse : Iterator a -> Iterator a`

Reverses an iterator.

### `subsequences : Iterator a -> Iterator (Iterator a)`

Generates all subsequences of an iterator.

`[1,2,3].to_iter.subsequences` is `[[], [3], [2], [2, 3], [1], [1, 3], [1, 2], [1, 2, 3]].to_iter.map(to_iter)`.

### `sum : [a : Additive] Iterator a -> a`

Calculates the sum of elements of an iterator.

### `take : I64 -> Iterator a -> Iterator a`

Takes at most n elements from an iterator.

### `take_while : (a -> Bool) -> Iterator a -> Iterator a`

Take elements of an iterator while a condition is satisfied.
TODO: add test

### `to_array : Iterator a -> Array a`

Convert an iterator to an array.

### `zip : Iterator b -> Iterator a -> Iterator (a, b)`

Zip two iterators.

## `namespace Std::LessThan`

### `max : [a : LessThan] a -> a -> a`

### `min : [a : LessThan] a -> a -> a`

## `namespace Std::LoopResult`

### `break_m : [m : Monad] r -> m (LoopResult s r)`

Make a break value wrapped in a monad.

This is used with `loop_m` function.

### `continue_m : [m : Monad] s -> m (LoopResult s r)`

Make a continue value wrapped in a monad.

This is used with `loop_m` function.

## `namespace Std::Monad`

### `flatten : [m : Monad] m (m a) -> m a`

Flattens a nested monadic action.

### `unless : [m : Monad] Bool -> m () -> m ()`

`unless(cond, act)` where `act` is a monadic value which returns `()` perfoms `act` only when `cond` is false.

### `when : [m : Monad] Bool -> m () -> m ()`

`when(cond, act)` where `act` is a monadic value which returns `()` perfoms `act` only when `cond` is true.

## `namespace Std::Option`

### `as_some_or : a -> Option a -> a`

Unwrap an option value if it is `some`, or returns given default value if it is `none`.

### `map_or : b -> (a -> b) -> Option a -> b`

Returns the provided default value if the option is none, or applies a function to the contained value if the option is some.

## `namespace Std::Path`

### `parse : String -> Option Path`

Parse a string.

## `namespace Std::Ptr`

### `add_offset : I64 -> Ptr -> Ptr`

Adds an offset to a pointer.

### `subtract_ptr : Ptr -> Ptr -> I64`

Subtracts two pointers.

Note that `x.subtract_ptr(y)` calculates `x - y`, so `subtract_ptr(x, y)` calculates `y - x`.

## `namespace Std::PunchedArray`

### `plug_in : a -> PunchedArray a -> Array a`

Plug in an element to a punched array to get back an array.

### `unsafe_punch : I64 -> Array a -> (PunchedArray a, a)`

Creates a punched array by moving out the element at the specified index.

NOTE: this function assumes that the given array is unique WITHOUT CHECKING.
The uniqueness of the array is ensured in the `Array::act` function.

## `namespace Std::Result`

### `unwrap : Result e o -> o`

Returns the containing value if the value is ok, or otherwise aborts the program.

## `namespace Std::String`

### `_get_c_str : String -> Ptr`

Get the null-terminated C string.

Note that in case the string is not used after call of this function, the returned pointer will be already released.

### `_unsafe_from_c_str : Array U8 -> String`

Create a string from C string (i.e., null-terminated byte array).

If the byte array doesn't include `\0`, this function causes undefined behavior.

### `_unsafe_from_c_str_ptr : Ptr -> String`

Create a `String` from a pointer to null-terminated C string.

If `ptr` is not pointing to a valid null-terminated C string, this function cause undefined behavior.

### `borrow_c_str : (Ptr -> a) -> String -> a`

Call a function with a null-terminated C string.

### `concat : String -> String -> String`

Concatenate two strings.

Note: Since `s1.concat(s2)` puts `s2` after `s1`, `concat(lhs, rhs)` puts `lhs` after `rhs`.

### `concat_iter : Iterator String -> String`

Concatenate an iterator of strings.

### `empty : I64 -> String`

Create an empty string, which is reserved for a length.

### `find : String -> I64 -> String -> Option I64`

`str.find(token, start_idx)` finds the index where `token` firstly appears in `str`, starting from `start_idx`.

Note that this function basically returns a number less than or equal to `start_idx`, but there is an exception:
`str.find("", start_idx)` with `start_idx >= str.get_size` returns `str.get_size`, not `start_idx`.

### `get_bytes : String -> Array U8`

Gets the byte array of a string, containing null-terminator.

### `get_first_byte : String -> Option U8`

Gets the first byte of a string. Returns none if the string is empty.

### `get_last_byte : String -> Option U8`

Gets the last byte of a string. Returns none if the string is empty.

### `get_size : String -> I64`

Gets the length of a string.

### `get_sub : I64 -> I64 -> String -> String`

`String` version of `Array::get_sub`.

### `is_empty : String -> Bool`

Returns if the string is empty or not.

### `join : String -> Iterator String -> String`

Joins strings by a separator.

### `pop_back_byte : String -> String`

Removes the last byte.
If the string is empty, this function does nothing.

### `split : String -> String -> Iterator String`

`str.split(sep)` splits `str` by `sep` into an iterator.
- If `sep` is empty, this function returns an infinite sequence of ""s.
- If `sep` is non-empty and `str` is empty, this function returns an iterator with a single element "".

### `strip_first_bytes : (U8 -> Bool) -> String -> String`

Removes the first byte of a string while it satisifies the specified condition.

### `strip_first_spaces : String -> String`

Removes leading whitespace characters.

### `strip_last_bytes : (U8 -> Bool) -> String -> String`

Removes the last byte of a string while it satisifies the specified condition.

### `strip_last_newlines : String -> String`

Removes newlines and carriage returns at the end of the string.

### `strip_last_spaces : String -> String`

Removes trailing whitespace characters.

### `strip_spaces : String -> String`

Strips leading and trailing whitespace characters.

## `namespace Std::U16`

### `bit_and : Std::U16 -> Std::U16 -> Std::U16`

Calculates bitwise AND of two values.

### `bit_or : Std::U16 -> Std::U16 -> Std::U16`

Calculates bitwise OR of two values.

### `bit_xor : Std::U16 -> Std::U16 -> Std::U16`

Calculates bitwise XOR of two values.

### `maximum : U16`

### `minimum : U16`

### `shift_left : Std::U16 -> Std::U16 -> Std::U16`

`v.shift_left(w)` shifts `v` to left by `w` bits.

### `shift_right : Std::U16 -> Std::U16 -> Std::U16`

`v.shift_right(w)` shifts `v` to right by `w` bits.

### `to_CChar : Std::U16 -> Std::I8`

Casts a value of `U16` into a value of `CChar`.

### `to_CDouble : Std::U16 -> Std::F64`

Casts a value of `U16` into a value of `CDouble`.

### `to_CFloat : Std::U16 -> Std::F32`

Casts a value of `U16` into a value of `CFloat`.

### `to_CInt : Std::U16 -> Std::I32`

Casts a value of `U16` into a value of `CInt`.

### `to_CLong : Std::U16 -> Std::I64`

Casts a value of `U16` into a value of `CLong`.

### `to_CLongLong : Std::U16 -> Std::I64`

Casts a value of `U16` into a value of `CLongLong`.

### `to_CShort : Std::U16 -> Std::I16`

Casts a value of `U16` into a value of `CShort`.

### `to_CSizeT : Std::U16 -> Std::U64`

Casts a value of `U16` into a value of `CSizeT`.

### `to_CUnsignedChar : Std::U16 -> Std::U8`

Casts a value of `U16` into a value of `CUnsignedChar`.

### `to_CUnsignedInt : Std::U16 -> Std::U32`

Casts a value of `U16` into a value of `CUnsignedInt`.

### `to_CUnsignedLong : Std::U16 -> Std::U64`

Casts a value of `U16` into a value of `CUnsignedLong`.

### `to_CUnsignedLongLong : Std::U16 -> Std::U64`

Casts a value of `U16` into a value of `CUnsignedLongLong`.

### `to_CUnsignedShort : Std::U16 -> Std::U16`

Casts a value of `U16` into a value of `CUnsignedShort`.

### `to_F32 : Std::U16 -> Std::F32`

Casts a value of `U16` into a value of `F32`.

### `to_F64 : Std::U16 -> Std::F64`

Casts a value of `U16` into a value of `F64`.

### `to_I16 : Std::U16 -> Std::I16`

Casts a value of `U16` into a value of `I16`.

### `to_I32 : Std::U16 -> Std::I32`

Casts a value of `U16` into a value of `I32`.

### `to_I64 : Std::U16 -> Std::I64`

Casts a value of `U16` into a value of `I64`.

### `to_I8 : Std::U16 -> Std::I8`

Casts a value of `U16` into a value of `I8`.

### `to_U16 : Std::U16 -> Std::U16`

Casts a value of `U16` into a value of `U16`.

### `to_U32 : Std::U16 -> Std::U32`

Casts a value of `U16` into a value of `U32`.

### `to_U64 : Std::U16 -> Std::U64`

Casts a value of `U16` into a value of `U64`.

### `to_U8 : Std::U16 -> Std::U8`

Casts a value of `U16` into a value of `U8`.

## `namespace Std::U32`

### `bit_and : Std::U32 -> Std::U32 -> Std::U32`

Calculates bitwise AND of two values.

### `bit_or : Std::U32 -> Std::U32 -> Std::U32`

Calculates bitwise OR of two values.

### `bit_xor : Std::U32 -> Std::U32 -> Std::U32`

Calculates bitwise XOR of two values.

### `maximum : U32`

### `minimum : U32`

### `shift_left : Std::U32 -> Std::U32 -> Std::U32`

`v.shift_left(w)` shifts `v` to left by `w` bits.

### `shift_right : Std::U32 -> Std::U32 -> Std::U32`

`v.shift_right(w)` shifts `v` to right by `w` bits.

### `to_CChar : Std::U32 -> Std::I8`

Casts a value of `U32` into a value of `CChar`.

### `to_CDouble : Std::U32 -> Std::F64`

Casts a value of `U32` into a value of `CDouble`.

### `to_CFloat : Std::U32 -> Std::F32`

Casts a value of `U32` into a value of `CFloat`.

### `to_CInt : Std::U32 -> Std::I32`

Casts a value of `U32` into a value of `CInt`.

### `to_CLong : Std::U32 -> Std::I64`

Casts a value of `U32` into a value of `CLong`.

### `to_CLongLong : Std::U32 -> Std::I64`

Casts a value of `U32` into a value of `CLongLong`.

### `to_CShort : Std::U32 -> Std::I16`

Casts a value of `U32` into a value of `CShort`.

### `to_CSizeT : Std::U32 -> Std::U64`

Casts a value of `U32` into a value of `CSizeT`.

### `to_CUnsignedChar : Std::U32 -> Std::U8`

Casts a value of `U32` into a value of `CUnsignedChar`.

### `to_CUnsignedInt : Std::U32 -> Std::U32`

Casts a value of `U32` into a value of `CUnsignedInt`.

### `to_CUnsignedLong : Std::U32 -> Std::U64`

Casts a value of `U32` into a value of `CUnsignedLong`.

### `to_CUnsignedLongLong : Std::U32 -> Std::U64`

Casts a value of `U32` into a value of `CUnsignedLongLong`.

### `to_CUnsignedShort : Std::U32 -> Std::U16`

Casts a value of `U32` into a value of `CUnsignedShort`.

### `to_F32 : Std::U32 -> Std::F32`

Casts a value of `U32` into a value of `F32`.

### `to_F64 : Std::U32 -> Std::F64`

Casts a value of `U32` into a value of `F64`.

### `to_I16 : Std::U32 -> Std::I16`

Casts a value of `U32` into a value of `I16`.

### `to_I32 : Std::U32 -> Std::I32`

Casts a value of `U32` into a value of `I32`.

### `to_I64 : Std::U32 -> Std::I64`

Casts a value of `U32` into a value of `I64`.

### `to_I8 : Std::U32 -> Std::I8`

Casts a value of `U32` into a value of `I8`.

### `to_U16 : Std::U32 -> Std::U16`

Casts a value of `U32` into a value of `U16`.

### `to_U32 : Std::U32 -> Std::U32`

Casts a value of `U32` into a value of `U32`.

### `to_U64 : Std::U32 -> Std::U64`

Casts a value of `U32` into a value of `U64`.

### `to_U8 : Std::U32 -> Std::U8`

Casts a value of `U32` into a value of `U8`.

## `namespace Std::U64`

### `bit_and : Std::U64 -> Std::U64 -> Std::U64`

Calculates bitwise AND of two values.

### `bit_or : Std::U64 -> Std::U64 -> Std::U64`

Calculates bitwise OR of two values.

### `bit_xor : Std::U64 -> Std::U64 -> Std::U64`

Calculates bitwise XOR of two values.

### `maximum : U64`

### `minimum : U64`

### `shift_left : Std::U64 -> Std::U64 -> Std::U64`

`v.shift_left(w)` shifts `v` to left by `w` bits.

### `shift_right : Std::U64 -> Std::U64 -> Std::U64`

`v.shift_right(w)` shifts `v` to right by `w` bits.

### `to_CChar : Std::U64 -> Std::I8`

Casts a value of `U64` into a value of `CChar`.

### `to_CDouble : Std::U64 -> Std::F64`

Casts a value of `U64` into a value of `CDouble`.

### `to_CFloat : Std::U64 -> Std::F32`

Casts a value of `U64` into a value of `CFloat`.

### `to_CInt : Std::U64 -> Std::I32`

Casts a value of `U64` into a value of `CInt`.

### `to_CLong : Std::U64 -> Std::I64`

Casts a value of `U64` into a value of `CLong`.

### `to_CLongLong : Std::U64 -> Std::I64`

Casts a value of `U64` into a value of `CLongLong`.

### `to_CShort : Std::U64 -> Std::I16`

Casts a value of `U64` into a value of `CShort`.

### `to_CSizeT : Std::U64 -> Std::U64`

Casts a value of `U64` into a value of `CSizeT`.

### `to_CUnsignedChar : Std::U64 -> Std::U8`

Casts a value of `U64` into a value of `CUnsignedChar`.

### `to_CUnsignedInt : Std::U64 -> Std::U32`

Casts a value of `U64` into a value of `CUnsignedInt`.

### `to_CUnsignedLong : Std::U64 -> Std::U64`

Casts a value of `U64` into a value of `CUnsignedLong`.

### `to_CUnsignedLongLong : Std::U64 -> Std::U64`

Casts a value of `U64` into a value of `CUnsignedLongLong`.

### `to_CUnsignedShort : Std::U64 -> Std::U16`

Casts a value of `U64` into a value of `CUnsignedShort`.

### `to_F32 : Std::U64 -> Std::F32`

Casts a value of `U64` into a value of `F32`.

### `to_F64 : Std::U64 -> Std::F64`

Casts a value of `U64` into a value of `F64`.

### `to_I16 : Std::U64 -> Std::I16`

Casts a value of `U64` into a value of `I16`.

### `to_I32 : Std::U64 -> Std::I32`

Casts a value of `U64` into a value of `I32`.

### `to_I64 : Std::U64 -> Std::I64`

Casts a value of `U64` into a value of `I64`.

### `to_I8 : Std::U64 -> Std::I8`

Casts a value of `U64` into a value of `I8`.

### `to_U16 : Std::U64 -> Std::U16`

Casts a value of `U64` into a value of `U16`.

### `to_U32 : Std::U64 -> Std::U32`

Casts a value of `U64` into a value of `U32`.

### `to_U64 : Std::U64 -> Std::U64`

Casts a value of `U64` into a value of `U64`.

### `to_U8 : Std::U64 -> Std::U8`

Casts a value of `U64` into a value of `U8`.

## `namespace Std::U8`

### `bit_and : Std::U8 -> Std::U8 -> Std::U8`

Calculates bitwise AND of two values.

### `bit_or : Std::U8 -> Std::U8 -> Std::U8`

Calculates bitwise OR of two values.

### `bit_xor : Std::U8 -> Std::U8 -> Std::U8`

Calculates bitwise XOR of two values.

### `maximum : U8`

### `minimum : U8`

### `shift_left : Std::U8 -> Std::U8 -> Std::U8`

`v.shift_left(w)` shifts `v` to left by `w` bits.

### `shift_right : Std::U8 -> Std::U8 -> Std::U8`

`v.shift_right(w)` shifts `v` to right by `w` bits.

### `to_CChar : Std::U8 -> Std::I8`

Casts a value of `U8` into a value of `CChar`.

### `to_CDouble : Std::U8 -> Std::F64`

Casts a value of `U8` into a value of `CDouble`.

### `to_CFloat : Std::U8 -> Std::F32`

Casts a value of `U8` into a value of `CFloat`.

### `to_CInt : Std::U8 -> Std::I32`

Casts a value of `U8` into a value of `CInt`.

### `to_CLong : Std::U8 -> Std::I64`

Casts a value of `U8` into a value of `CLong`.

### `to_CLongLong : Std::U8 -> Std::I64`

Casts a value of `U8` into a value of `CLongLong`.

### `to_CShort : Std::U8 -> Std::I16`

Casts a value of `U8` into a value of `CShort`.

### `to_CSizeT : Std::U8 -> Std::U64`

Casts a value of `U8` into a value of `CSizeT`.

### `to_CUnsignedChar : Std::U8 -> Std::U8`

Casts a value of `U8` into a value of `CUnsignedChar`.

### `to_CUnsignedInt : Std::U8 -> Std::U32`

Casts a value of `U8` into a value of `CUnsignedInt`.

### `to_CUnsignedLong : Std::U8 -> Std::U64`

Casts a value of `U8` into a value of `CUnsignedLong`.

### `to_CUnsignedLongLong : Std::U8 -> Std::U64`

Casts a value of `U8` into a value of `CUnsignedLongLong`.

### `to_CUnsignedShort : Std::U8 -> Std::U16`

Casts a value of `U8` into a value of `CUnsignedShort`.

### `to_F32 : Std::U8 -> Std::F32`

Casts a value of `U8` into a value of `F32`.

### `to_F64 : Std::U8 -> Std::F64`

Casts a value of `U8` into a value of `F64`.

### `to_I16 : Std::U8 -> Std::I16`

Casts a value of `U8` into a value of `I16`.

### `to_I32 : Std::U8 -> Std::I32`

Casts a value of `U8` into a value of `I32`.

### `to_I64 : Std::U8 -> Std::I64`

Casts a value of `U8` into a value of `I64`.

### `to_I8 : Std::U8 -> Std::I8`

Casts a value of `U8` into a value of `I8`.

### `to_U16 : Std::U8 -> Std::U16`

Casts a value of `U8` into a value of `U16`.

### `to_U32 : Std::U8 -> Std::U32`

Casts a value of `U8` into a value of `U32`.

### `to_U64 : Std::U8 -> Std::U64`

Casts a value of `U8` into a value of `U64`.

### `to_U8 : Std::U8 -> Std::U8`

Casts a value of `U8` into a value of `U8`.