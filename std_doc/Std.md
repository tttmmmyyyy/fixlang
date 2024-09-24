- [`module Std`](#module-std)
- [Types and aliases](#types-and-aliases)
  - [`namespace Std`](#namespace-std)
    - [`type Array a = box { primitive }`](#type-array-a--box--primitive-)
    - [`type Bool = unbox { primitive }`](#type-bool--unbox--primitive-)
    - [`type Boxed a = box struct { ...fields... }`](#type-boxed-a--box-struct--fields-)
      - [field `value : a`](#field-value--a)
    - [`type ErrMsg = Std::String`](#type-errmsg--stdstring)
    - [`type F32 = unbox { primitive }`](#type-f32--unbox--primitive-)
    - [`type F64 = unbox { primitive }`](#type-f64--unbox--primitive-)
    - [`type I16 = unbox { primitive }`](#type-i16--unbox--primitive-)
    - [`type I32 = unbox { primitive }`](#type-i32--unbox--primitive-)
    - [`type I64 = unbox { primitive }`](#type-i64--unbox--primitive-)
    - [`type I8 = unbox { primitive }`](#type-i8--unbox--primitive-)
    - [`type IO a = unbox struct { ...fields... }`](#type-io-a--unbox-struct--fields-)
      - [field `_data : () -> a`](#field-_data-----a)
    - [`type Iterator a = unbox struct { ...fields... }`](#type-iterator-a--unbox-struct--fields-)
      - [field `next : () -> Std::Option (a, Std::Iterator a)`](#field-next-----stdoption-a-stditerator-a)
    - [`type Lazy = () -> a`](#type-lazy-----a)
    - [`type LoopResult s b = unbox union { ...variants... }`](#type-loopresult-s-b--unbox-union--variants-)
      - [variant `continue : s`](#variant-continue--s)
      - [variant `break : b`](#variant-break--b)
    - [`type Option a = unbox union { ...variants... }`](#type-option-a--unbox-union--variants-)
      - [variant `none : ()`](#variant-none--)
      - [variant `some : a`](#variant-some--a)
    - [`type Path = unbox struct { ...fields... }`](#type-path--unbox-struct--fields-)
      - [field `_data : Std::String`](#field-_data--stdstring)
    - [`type Ptr = unbox { primitive }`](#type-ptr--unbox--primitive-)
    - [`type PunchedArray a = unbox struct { ...fields... }`](#type-punchedarray-a--unbox-struct--fields-)
      - [field `_data : Std::FFI::Destructor (Std::Array a)`](#field-_data--stdffidestructor-stdarray-a)
      - [field `idx : Std::I64`](#field-idx--stdi64)
    - [`type Result e o = unbox union { ...variants... }`](#type-result-e-o--unbox-union--variants-)
      - [variant `ok : o`](#variant-ok--o)
      - [variant `err : e`](#variant-err--e)
    - [`type String = unbox struct { ...fields... }`](#type-string--unbox-struct--fields-)
      - [field `_data : Std::Array Std::U8`](#field-_data--stdarray-stdu8)
    - [`type Tuple0 = unbox struct { ...fields... }`](#type-tuple0--unbox-struct--fields-)
    - [`type Tuple2 t0 t1 = unbox struct { ...fields... }`](#type-tuple2-t0-t1--unbox-struct--fields-)
      - [field `0 : t0`](#field-0--t0)
      - [field `1 : t1`](#field-1--t1)
    - [`type Tuple3 t0 t1 t2 = unbox struct { ...fields... }`](#type-tuple3-t0-t1-t2--unbox-struct--fields-)
      - [field `0 : t0`](#field-0--t0-1)
      - [field `1 : t1`](#field-1--t1-1)
      - [field `2 : t2`](#field-2--t2)
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
      - [field `_data : Std::IO (Std::Result Std::String a)`](#field-_data--stdio-stdresult-stdstring-a)
    - [`type IOHandle = unbox struct { ...fields... }`](#type-iohandle--unbox-struct--fields-)
      - [field `_data : Std::FFI::Destructor Std::Ptr`](#field-_data--stdffidestructor-stdptr)
- [Traits and aliases](#traits-and-aliases)
  - [`namespace Std`](#namespace-std-1)
    - [`trait a : Add`](#trait-a--add)
      - [method `add : a -> a -> a`](#method-add--a---a---a)
    - [`trait a : Div`](#trait-a--div)
      - [method `div : a -> a -> a`](#method-div--a---a---a)
    - [`trait a : Eq`](#trait-a--eq)
      - [method `eq : a -> a -> Std::Bool`](#method-eq--a---a---stdbool)
    - [`trait a : FromBytes`](#trait-a--frombytes)
      - [method `from_bytes : Std::Array Std::U8 -> Std::Result Std::String a`](#method-from_bytes--stdarray-stdu8---stdresult-stdstring-a)
    - [`trait a : FromString`](#trait-a--fromstring)
      - [method `from_string : Std::String -> Std::Result Std::String a`](#method-from_string--stdstring---stdresult-stdstring-a)
    - [`trait [f : *->*] f : Functor`](#trait-f----f--functor)
      - [method `map : (a -> b) -> f a -> f b`](#method-map--a---b---f-a---f-b)
    - [`trait a : LessThan`](#trait-a--lessthan)
      - [method `less_than : a -> a -> Std::Bool`](#method-less_than--a---a---stdbool)
    - [`trait a : LessThanOrEq`](#trait-a--lessthanoreq)
      - [method `less_than_or_eq : a -> a -> Std::Bool`](#method-less_than_or_eq--a---a---stdbool)
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
      - [method `to_bytes : a -> Std::Array Std::U8`](#method-to_bytes--a---stdarray-stdu8)
    - [`trait a : ToString`](#trait-a--tostring)
      - [method `to_string : a -> Std::String`](#method-to_string--a---stdstring)
    - [`trait a : Zero`](#trait-a--zero)
      - [method `zero : a`](#method-zero--a)
- [Trait implementations](#trait-implementations)
    - [`impl () : Std::Eq`](#impl---stdeq)
    - [`impl () : Std::ToString`](#impl---stdtostring)
    - [`impl (t0, *) : Std::Functor`](#impl-t0---stdfunctor)
    - [`impl [t0 : Std::Eq, t1 : Std::Eq] (t0, t1) : Std::Eq`](#impl-t0--stdeq-t1--stdeq-t0-t1--stdeq)
    - [`impl [t0 : Std::Eq, t0 : Std::LessThan, t1 : Std::Eq, t1 : Std::LessThan] (t0, t1) : Std::LessThan`](#impl-t0--stdeq-t0--stdlessthan-t1--stdeq-t1--stdlessthan-t0-t1--stdlessthan)
    - [`impl [t0 : Std::Eq, t0 : Std::LessThanOrEq, t1 : Std::Eq, t1 : Std::LessThanOrEq] (t0, t1) : Std::LessThanOrEq`](#impl-t0--stdeq-t0--stdlessthanoreq-t1--stdeq-t1--stdlessthanoreq-t0-t1--stdlessthanoreq)
    - [`impl [t0 : Std::ToString, t1 : Std::ToString] (t0, t1) : Std::ToString`](#impl-t0--stdtostring-t1--stdtostring-t0-t1--stdtostring)
    - [`impl (t0, t1, *) : Std::Functor`](#impl-t0-t1---stdfunctor)
    - [`impl [t0 : Std::Eq, t1 : Std::Eq, t2 : Std::Eq] (t0, t1, t2) : Std::Eq`](#impl-t0--stdeq-t1--stdeq-t2--stdeq-t0-t1-t2--stdeq)
    - [`impl [t0 : Std::Eq, t0 : Std::LessThan, t1 : Std::Eq, t1 : Std::LessThan, t2 : Std::Eq, t2 : Std::LessThan] (t0, t1, t2) : Std::LessThan`](#impl-t0--stdeq-t0--stdlessthan-t1--stdeq-t1--stdlessthan-t2--stdeq-t2--stdlessthan-t0-t1-t2--stdlessthan)
    - [`impl [t0 : Std::Eq, t0 : Std::LessThanOrEq, t1 : Std::Eq, t1 : Std::LessThanOrEq, t2 : Std::Eq, t2 : Std::LessThanOrEq] (t0, t1, t2) : Std::LessThanOrEq`](#impl-t0--stdeq-t0--stdlessthanoreq-t1--stdeq-t1--stdlessthanoreq-t2--stdeq-t2--stdlessthanoreq-t0-t1-t2--stdlessthanoreq)
    - [`impl [t0 : Std::ToString, t1 : Std::ToString, t2 : Std::ToString] (t0, t1, t2) : Std::ToString`](#impl-t0--stdtostring-t1--stdtostring-t2--stdtostring-t0-t1-t2--stdtostring)
    - [`impl Std::Array : Std::Functor`](#impl-stdarray--stdfunctor)
    - [`impl Std::Array : Std::Monad`](#impl-stdarray--stdmonad)
    - [`impl [a : Std::Eq] Std::Array a : Std::Eq`](#impl-a--stdeq-stdarray-a--stdeq)
    - [`impl [a : Std::Eq, a : Std::LessThan] Std::Array a : Std::LessThan`](#impl-a--stdeq-a--stdlessthan-stdarray-a--stdlessthan)
    - [`impl [a : Std::Eq, a : Std::LessThanOrEq] Std::Array a : Std::LessThanOrEq`](#impl-a--stdeq-a--stdlessthanoreq-stdarray-a--stdlessthanoreq)
    - [`impl [a : Std::ToString] Std::Array a : Std::ToString`](#impl-a--stdtostring-stdarray-a--stdtostring)
    - [`impl Std::Bool : Std::Eq`](#impl-stdbool--stdeq)
    - [`impl Std::Bool : Std::Not`](#impl-stdbool--stdnot)
    - [`impl Std::Bool : Std::ToString`](#impl-stdbool--stdtostring)
    - [`impl Std::F32 : Std::Add`](#impl-stdf32--stdadd)
    - [`impl Std::F32 : Std::Div`](#impl-stdf32--stddiv)
    - [`impl Std::F32 : Std::Eq`](#impl-stdf32--stdeq)
    - [`impl Std::F32 : Std::FromBytes`](#impl-stdf32--stdfrombytes)
    - [`impl Std::F32 : Std::FromString`](#impl-stdf32--stdfromstring)
    - [`impl Std::F32 : Std::LessThan`](#impl-stdf32--stdlessthan)
    - [`impl Std::F32 : Std::LessThanOrEq`](#impl-stdf32--stdlessthanoreq)
    - [`impl Std::F32 : Std::Mul`](#impl-stdf32--stdmul)
    - [`impl Std::F32 : Std::Neg`](#impl-stdf32--stdneg)
    - [`impl Std::F32 : Std::Sub`](#impl-stdf32--stdsub)
    - [`impl Std::F32 : Std::ToBytes`](#impl-stdf32--stdtobytes)
    - [`impl Std::F32 : Std::ToString`](#impl-stdf32--stdtostring)
    - [`impl Std::F32 : Std::Zero`](#impl-stdf32--stdzero)
    - [`impl Std::F64 : Std::Add`](#impl-stdf64--stdadd)
    - [`impl Std::F64 : Std::Div`](#impl-stdf64--stddiv)
    - [`impl Std::F64 : Std::Eq`](#impl-stdf64--stdeq)
    - [`impl Std::F64 : Std::FromBytes`](#impl-stdf64--stdfrombytes)
    - [`impl Std::F64 : Std::FromString`](#impl-stdf64--stdfromstring)
    - [`impl Std::F64 : Std::LessThan`](#impl-stdf64--stdlessthan)
    - [`impl Std::F64 : Std::LessThanOrEq`](#impl-stdf64--stdlessthanoreq)
    - [`impl Std::F64 : Std::Mul`](#impl-stdf64--stdmul)
    - [`impl Std::F64 : Std::Neg`](#impl-stdf64--stdneg)
    - [`impl Std::F64 : Std::Sub`](#impl-stdf64--stdsub)
    - [`impl Std::F64 : Std::ToBytes`](#impl-stdf64--stdtobytes)
    - [`impl Std::F64 : Std::ToString`](#impl-stdf64--stdtostring)
    - [`impl Std::F64 : Std::Zero`](#impl-stdf64--stdzero)
    - [`impl Std::I16 : Std::Add`](#impl-stdi16--stdadd)
    - [`impl Std::I16 : Std::Div`](#impl-stdi16--stddiv)
    - [`impl Std::I16 : Std::Eq`](#impl-stdi16--stdeq)
    - [`impl Std::I16 : Std::FromBytes`](#impl-stdi16--stdfrombytes)
    - [`impl Std::I16 : Std::FromString`](#impl-stdi16--stdfromstring)
    - [`impl Std::I16 : Std::LessThan`](#impl-stdi16--stdlessthan)
    - [`impl Std::I16 : Std::LessThanOrEq`](#impl-stdi16--stdlessthanoreq)
    - [`impl Std::I16 : Std::Mul`](#impl-stdi16--stdmul)
    - [`impl Std::I16 : Std::Neg`](#impl-stdi16--stdneg)
    - [`impl Std::I16 : Std::Rem`](#impl-stdi16--stdrem)
    - [`impl Std::I16 : Std::Sub`](#impl-stdi16--stdsub)
    - [`impl Std::I16 : Std::ToBytes`](#impl-stdi16--stdtobytes)
    - [`impl Std::I16 : Std::ToString`](#impl-stdi16--stdtostring)
    - [`impl Std::I16 : Std::Zero`](#impl-stdi16--stdzero)
    - [`impl Std::I32 : Std::Add`](#impl-stdi32--stdadd)
    - [`impl Std::I32 : Std::Div`](#impl-stdi32--stddiv)
    - [`impl Std::I32 : Std::Eq`](#impl-stdi32--stdeq)
    - [`impl Std::I32 : Std::FromBytes`](#impl-stdi32--stdfrombytes)
    - [`impl Std::I32 : Std::FromString`](#impl-stdi32--stdfromstring)
    - [`impl Std::I32 : Std::LessThan`](#impl-stdi32--stdlessthan)
    - [`impl Std::I32 : Std::LessThanOrEq`](#impl-stdi32--stdlessthanoreq)
    - [`impl Std::I32 : Std::Mul`](#impl-stdi32--stdmul)
    - [`impl Std::I32 : Std::Neg`](#impl-stdi32--stdneg)
    - [`impl Std::I32 : Std::Rem`](#impl-stdi32--stdrem)
    - [`impl Std::I32 : Std::Sub`](#impl-stdi32--stdsub)
    - [`impl Std::I32 : Std::ToBytes`](#impl-stdi32--stdtobytes)
    - [`impl Std::I32 : Std::ToString`](#impl-stdi32--stdtostring)
    - [`impl Std::I32 : Std::Zero`](#impl-stdi32--stdzero)
    - [`impl Std::I64 : Std::Add`](#impl-stdi64--stdadd)
    - [`impl Std::I64 : Std::Div`](#impl-stdi64--stddiv)
    - [`impl Std::I64 : Std::Eq`](#impl-stdi64--stdeq)
    - [`impl Std::I64 : Std::FromBytes`](#impl-stdi64--stdfrombytes)
    - [`impl Std::I64 : Std::FromString`](#impl-stdi64--stdfromstring)
    - [`impl Std::I64 : Std::LessThan`](#impl-stdi64--stdlessthan)
    - [`impl Std::I64 : Std::LessThanOrEq`](#impl-stdi64--stdlessthanoreq)
    - [`impl Std::I64 : Std::Mul`](#impl-stdi64--stdmul)
    - [`impl Std::I64 : Std::Neg`](#impl-stdi64--stdneg)
    - [`impl Std::I64 : Std::Rem`](#impl-stdi64--stdrem)
    - [`impl Std::I64 : Std::Sub`](#impl-stdi64--stdsub)
    - [`impl Std::I64 : Std::ToBytes`](#impl-stdi64--stdtobytes)
    - [`impl Std::I64 : Std::ToString`](#impl-stdi64--stdtostring)
    - [`impl Std::I64 : Std::Zero`](#impl-stdi64--stdzero)
    - [`impl Std::I8 : Std::Add`](#impl-stdi8--stdadd)
    - [`impl Std::I8 : Std::Div`](#impl-stdi8--stddiv)
    - [`impl Std::I8 : Std::Eq`](#impl-stdi8--stdeq)
    - [`impl Std::I8 : Std::FromBytes`](#impl-stdi8--stdfrombytes)
    - [`impl Std::I8 : Std::FromString`](#impl-stdi8--stdfromstring)
    - [`impl Std::I8 : Std::LessThan`](#impl-stdi8--stdlessthan)
    - [`impl Std::I8 : Std::LessThanOrEq`](#impl-stdi8--stdlessthanoreq)
    - [`impl Std::I8 : Std::Mul`](#impl-stdi8--stdmul)
    - [`impl Std::I8 : Std::Neg`](#impl-stdi8--stdneg)
    - [`impl Std::I8 : Std::Rem`](#impl-stdi8--stdrem)
    - [`impl Std::I8 : Std::Sub`](#impl-stdi8--stdsub)
    - [`impl Std::I8 : Std::ToBytes`](#impl-stdi8--stdtobytes)
    - [`impl Std::I8 : Std::ToString`](#impl-stdi8--stdtostring)
    - [`impl Std::I8 : Std::Zero`](#impl-stdi8--stdzero)
    - [`impl Std::IO : Std::Functor`](#impl-stdio--stdfunctor)
    - [`impl Std::IO : Std::Monad`](#impl-stdio--stdmonad)
    - [`impl Std::IO::IOFail : Std::Functor`](#impl-stdioiofail--stdfunctor)
    - [`impl Std::IO::IOFail : Std::Monad`](#impl-stdioiofail--stdmonad)
    - [`impl Std::Iterator : Std::Functor`](#impl-stditerator--stdfunctor)
    - [`impl Std::Iterator : Std::Monad`](#impl-stditerator--stdmonad)
    - [`impl Std::Iterator a : Std::Add`](#impl-stditerator-a--stdadd)
    - [`impl [a : Std::Eq] Std::Iterator a : Std::Eq`](#impl-a--stdeq-stditerator-a--stdeq)
    - [`impl Std::Option : Std::Functor`](#impl-stdoption--stdfunctor)
    - [`impl Std::Option : Std::Monad`](#impl-stdoption--stdmonad)
    - [`impl [a : Std::Eq] Std::Option a : Std::Eq`](#impl-a--stdeq-stdoption-a--stdeq)
    - [`impl [a : Std::ToString] Std::Option a : Std::ToString`](#impl-a--stdtostring-stdoption-a--stdtostring)
    - [`impl Std::Path : Std::ToString`](#impl-stdpath--stdtostring)
    - [`impl Std::Ptr : Std::Eq`](#impl-stdptr--stdeq)
    - [`impl Std::Ptr : Std::ToString`](#impl-stdptr--stdtostring)
    - [`impl Std::Result e : Std::Functor`](#impl-stdresult-e--stdfunctor)
    - [`impl Std::Result e : Std::Monad`](#impl-stdresult-e--stdmonad)
    - [`impl [e : Std::Eq, a : Std::Eq] Std::Result e a : Std::Eq`](#impl-e--stdeq-a--stdeq-stdresult-e-a--stdeq)
    - [`impl [e : Std::ToString, a : Std::ToString] Std::Result e a : Std::ToString`](#impl-e--stdtostring-a--stdtostring-stdresult-e-a--stdtostring)
    - [`impl Std::String : Std::Add`](#impl-stdstring--stdadd)
    - [`impl Std::String : Std::Eq`](#impl-stdstring--stdeq)
    - [`impl Std::String : Std::LessThan`](#impl-stdstring--stdlessthan)
    - [`impl Std::String : Std::LessThanOrEq`](#impl-stdstring--stdlessthanoreq)
    - [`impl Std::String : Std::ToString`](#impl-stdstring--stdtostring)
    - [`impl Std::U16 : Std::Add`](#impl-stdu16--stdadd)
    - [`impl Std::U16 : Std::Div`](#impl-stdu16--stddiv)
    - [`impl Std::U16 : Std::Eq`](#impl-stdu16--stdeq)
    - [`impl Std::U16 : Std::FromBytes`](#impl-stdu16--stdfrombytes)
    - [`impl Std::U16 : Std::FromString`](#impl-stdu16--stdfromstring)
    - [`impl Std::U16 : Std::LessThan`](#impl-stdu16--stdlessthan)
    - [`impl Std::U16 : Std::LessThanOrEq`](#impl-stdu16--stdlessthanoreq)
    - [`impl Std::U16 : Std::Mul`](#impl-stdu16--stdmul)
    - [`impl Std::U16 : Std::Neg`](#impl-stdu16--stdneg)
    - [`impl Std::U16 : Std::Rem`](#impl-stdu16--stdrem)
    - [`impl Std::U16 : Std::Sub`](#impl-stdu16--stdsub)
    - [`impl Std::U16 : Std::ToBytes`](#impl-stdu16--stdtobytes)
    - [`impl Std::U16 : Std::ToString`](#impl-stdu16--stdtostring)
    - [`impl Std::U16 : Std::Zero`](#impl-stdu16--stdzero)
    - [`impl Std::U32 : Std::Add`](#impl-stdu32--stdadd)
    - [`impl Std::U32 : Std::Div`](#impl-stdu32--stddiv)
    - [`impl Std::U32 : Std::Eq`](#impl-stdu32--stdeq)
    - [`impl Std::U32 : Std::FromBytes`](#impl-stdu32--stdfrombytes)
    - [`impl Std::U32 : Std::FromString`](#impl-stdu32--stdfromstring)
    - [`impl Std::U32 : Std::LessThan`](#impl-stdu32--stdlessthan)
    - [`impl Std::U32 : Std::LessThanOrEq`](#impl-stdu32--stdlessthanoreq)
    - [`impl Std::U32 : Std::Mul`](#impl-stdu32--stdmul)
    - [`impl Std::U32 : Std::Neg`](#impl-stdu32--stdneg)
    - [`impl Std::U32 : Std::Rem`](#impl-stdu32--stdrem)
    - [`impl Std::U32 : Std::Sub`](#impl-stdu32--stdsub)
    - [`impl Std::U32 : Std::ToBytes`](#impl-stdu32--stdtobytes)
    - [`impl Std::U32 : Std::ToString`](#impl-stdu32--stdtostring)
    - [`impl Std::U32 : Std::Zero`](#impl-stdu32--stdzero)
    - [`impl Std::U64 : Std::Add`](#impl-stdu64--stdadd)
    - [`impl Std::U64 : Std::Div`](#impl-stdu64--stddiv)
    - [`impl Std::U64 : Std::Eq`](#impl-stdu64--stdeq)
    - [`impl Std::U64 : Std::FromBytes`](#impl-stdu64--stdfrombytes)
    - [`impl Std::U64 : Std::FromString`](#impl-stdu64--stdfromstring)
    - [`impl Std::U64 : Std::LessThan`](#impl-stdu64--stdlessthan)
    - [`impl Std::U64 : Std::LessThanOrEq`](#impl-stdu64--stdlessthanoreq)
    - [`impl Std::U64 : Std::Mul`](#impl-stdu64--stdmul)
    - [`impl Std::U64 : Std::Neg`](#impl-stdu64--stdneg)
    - [`impl Std::U64 : Std::Rem`](#impl-stdu64--stdrem)
    - [`impl Std::U64 : Std::Sub`](#impl-stdu64--stdsub)
    - [`impl Std::U64 : Std::ToBytes`](#impl-stdu64--stdtobytes)
    - [`impl Std::U64 : Std::ToString`](#impl-stdu64--stdtostring)
    - [`impl Std::U64 : Std::Zero`](#impl-stdu64--stdzero)
    - [`impl Std::U8 : Std::Add`](#impl-stdu8--stdadd)
    - [`impl Std::U8 : Std::Div`](#impl-stdu8--stddiv)
    - [`impl Std::U8 : Std::Eq`](#impl-stdu8--stdeq)
    - [`impl Std::U8 : Std::FromBytes`](#impl-stdu8--stdfrombytes)
    - [`impl Std::U8 : Std::FromString`](#impl-stdu8--stdfromstring)
    - [`impl Std::U8 : Std::LessThan`](#impl-stdu8--stdlessthan)
    - [`impl Std::U8 : Std::LessThanOrEq`](#impl-stdu8--stdlessthanoreq)
    - [`impl Std::U8 : Std::Mul`](#impl-stdu8--stdmul)
    - [`impl Std::U8 : Std::Neg`](#impl-stdu8--stdneg)
    - [`impl Std::U8 : Std::Rem`](#impl-stdu8--stdrem)
    - [`impl Std::U8 : Std::Sub`](#impl-stdu8--stdsub)
    - [`impl Std::U8 : Std::ToBytes`](#impl-stdu8--stdtobytes)
    - [`impl Std::U8 : Std::ToString`](#impl-stdu8--stdtostring)
    - [`impl Std::U8 : Std::Zero`](#impl-stdu8--stdzero)
- [Values](#values)
  - [`namespace Std`](#namespace-std-2)
    - [`compose : (a -> b) -> (b -> c) -> a -> c`](#compose--a---b---b---c---a---c)
    - [`fix : ((a -> b) -> a -> b) -> a -> b`](#fix--a---b---a---b---a---b)
    - [`loop : s -> (s -> Std::LoopResult s b) -> b`](#loop--s---s---stdloopresult-s-b---b)
    - [`loop_m : [m : Std::Monad] s -> (s -> m (Std::LoopResult s r)) -> m r`](#loop_m--m--stdmonad-s---s---m-stdloopresult-s-r---m-r)
    - [`mark_threaded : a -> a`](#mark_threaded--a---a)
    - [`undefined : () -> a`](#undefined-----a)
    - [`unsafe_is_unique : a -> (Std::Bool, a)`](#unsafe_is_unique--a---stdbool-a)
  - [`namespace Std::Add`](#namespace-stdadd)
    - [`add : [a : Std::Add] a -> a -> a`](#add--a--stdadd-a---a---a)
  - [`namespace Std::Array`](#namespace-stdarray)
    - [`@ : Std::I64 -> Std::Array a -> a`](#--stdi64---stdarray-a---a)
    - [`_get_ptr : Std::Array a -> Std::Ptr`](#_get_ptr--stdarray-a---stdptr)
    - [`_get_sub_size_asif : Std::I64 -> Std::I64 -> Std::I64 -> Std::I64 -> Std::Array a -> Std::Array a`](#_get_sub_size_asif--stdi64---stdi64---stdi64---stdi64---stdarray-a---stdarray-a)
    - [`_sort_range_using_buffer : Std::Array a -> Std::I64 -> Std::I64 -> ((a, a) -> Std::Bool) -> Std::Array a -> (Std::Array a, Std::Array a)`](#_sort_range_using_buffer--stdarray-a---stdi64---stdi64---a-a---stdbool---stdarray-a---stdarray-a-stdarray-a)
    - [`_unsafe_get : Std::I64 -> Std::Array a -> a`](#_unsafe_get--stdi64---stdarray-a---a)
    - [`_unsafe_set : Std::I64 -> a -> Std::Array a -> Std::Array a`](#_unsafe_set--stdi64---a---stdarray-a---stdarray-a)
    - [`_unsafe_set_size : Std::I64 -> Std::Array a -> Std::Array a`](#_unsafe_set_size--stdi64---stdarray-a---stdarray-a)
    - [`act : [f : Std::Functor] Std::I64 -> (a -> f a) -> Std::Array a -> f (Std::Array a)`](#act--f--stdfunctor-stdi64---a---f-a---stdarray-a---f-stdarray-a)
    - [`append : Std::Array a -> Std::Array a -> Std::Array a`](#append--stdarray-a---stdarray-a---stdarray-a)
    - [`borrow_ptr : (Std::Ptr -> b) -> Std::Array a -> b`](#borrow_ptr--stdptr---b---stdarray-a---b)
    - [`empty : Std::I64 -> Std::Array a`](#empty--stdi64---stdarray-a)
    - [`fill : Std::I64 -> a -> Std::Array a`](#fill--stdi64---a---stdarray-a)
    - [`find_by : (a -> Std::Bool) -> Std::Array a -> Std::Option Std::I64`](#find_by--a---stdbool---stdarray-a---stdoption-stdi64)
    - [`force_unique : Std::Array a -> Std::Array a`](#force_unique--stdarray-a---stdarray-a)
    - [`from_iter : Std::Iterator a -> Std::Array a`](#from_iter--stditerator-a---stdarray-a)
    - [`from_map : Std::I64 -> (Std::I64 -> a) -> Std::Array a`](#from_map--stdi64---stdi64---a---stdarray-a)
    - [`get_capacity : Std::Array a -> Std::I64`](#get_capacity--stdarray-a---stdi64)
    - [`get_first : Std::Array a -> Std::Option a`](#get_first--stdarray-a---stdoption-a)
    - [`get_last : Std::Array a -> Std::Option a`](#get_last--stdarray-a---stdoption-a)
    - [`get_size : Std::Array a -> Std::I64`](#get_size--stdarray-a---stdi64)
    - [`get_sub : Std::I64 -> Std::I64 -> Std::Array a -> Std::Array a`](#get_sub--stdi64---stdi64---stdarray-a---stdarray-a)
    - [`is_empty : Std::Array a -> Std::Bool`](#is_empty--stdarray-a---stdbool)
    - [`mod : Std::I64 -> (a -> a) -> Std::Array a -> Std::Array a`](#mod--stdi64---a---a---stdarray-a---stdarray-a)
    - [`pop_back : Std::Array a -> Std::Array a`](#pop_back--stdarray-a---stdarray-a)
    - [`push_back : a -> Std::Array a -> Std::Array a`](#push_back--a---stdarray-a---stdarray-a)
    - [`reserve : Std::I64 -> Std::Array a -> Std::Array a`](#reserve--stdi64---stdarray-a---stdarray-a)
    - [`set : Std::I64 -> a -> Std::Array a -> Std::Array a`](#set--stdi64---a---stdarray-a---stdarray-a)
    - [`sort_by : ((a, a) -> Std::Bool) -> Std::Array a -> Std::Array a`](#sort_by--a-a---stdbool---stdarray-a---stdarray-a)
    - [`to_iter : Std::Array a -> Std::Iterator a`](#to_iter--stdarray-a---stditerator-a)
    - [`truncate : Std::I64 -> Std::Array a -> Std::Array a`](#truncate--stdi64---stdarray-a---stdarray-a)
  - [`namespace Std::Boxed`](#namespace-stdboxed)
    - [`@value : Std::Boxed a -> a`](#value--stdboxed-a---a)
    - [`act_value : [f : Std::Functor] (a -> f a) -> Std::Boxed a -> f (Std::Boxed a)`](#act_value--f--stdfunctor-a---f-a---stdboxed-a---f-stdboxed-a)
    - [`mod_value : (a -> a) -> Std::Boxed a -> Std::Boxed a`](#mod_value--a---a---stdboxed-a---stdboxed-a)
    - [`set_value : a -> Std::Boxed a -> Std::Boxed a`](#set_value--a---stdboxed-a---stdboxed-a)
  - [`namespace Std::Debug`](#namespace-stddebug)
    - [`_debug_print_to_stream : Std::IO::IOHandle -> Std::String -> ()`](#_debug_print_to_stream--stdioiohandle---stdstring---)
    - [`assert : (() -> Std::String) -> Std::Bool -> ()`](#assert-----stdstring---stdbool---)
    - [`assert_eq : [a : Std::Eq] (() -> Std::String) -> a -> a -> ()`](#assert_eq--a--stdeq----stdstring---a---a---)
    - [`assert_unique : (() -> Std::String) -> a -> a`](#assert_unique-----stdstring---a---a)
    - [`consumed_time_while_io : Std::IO a -> Std::IO (a, Std::F64)`](#consumed_time_while_io--stdio-a---stdio-a-stdf64)
    - [`consumed_time_while_lazy : (() -> a) -> (a, Std::F64)`](#consumed_time_while_lazy-----a---a-stdf64)
    - [`debug_eprint : Std::String -> ()`](#debug_eprint--stdstring---)
    - [`debug_eprintln : Std::String -> ()`](#debug_eprintln--stdstring---)
    - [`debug_print : Std::String -> ()`](#debug_print--stdstring---)
    - [`debug_println : Std::String -> ()`](#debug_println--stdstring---)
  - [`namespace Std::Div`](#namespace-stddiv)
    - [`div : [a : Std::Div] a -> a -> a`](#div--a--stddiv-a---a---a)
  - [`namespace Std::Eq`](#namespace-stdeq)
    - [`eq : [a : Std::Eq] a -> a -> Std::Bool`](#eq--a--stdeq-a---a---stdbool)
  - [`namespace Std::F32`](#namespace-stdf32)
    - [`abs : Std::F32 -> Std::F32`](#abs--stdf32---stdf32)
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
    - [`to_string_exp : Std::F32 -> Std::String`](#to_string_exp--stdf32---stdstring)
    - [`to_string_exp_precision : Std::U8 -> Std::F32 -> Std::String`](#to_string_exp_precision--stdu8---stdf32---stdstring)
    - [`to_string_precision : Std::U8 -> Std::F32 -> Std::String`](#to_string_precision--stdu8---stdf32---stdstring)
  - [`namespace Std::F64`](#namespace-stdf64)
    - [`abs : Std::F64 -> Std::F64`](#abs--stdf64---stdf64)
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
    - [`to_string_exp : Std::F64 -> Std::String`](#to_string_exp--stdf64---stdstring)
    - [`to_string_exp_precision : Std::U8 -> Std::F64 -> Std::String`](#to_string_exp_precision--stdu8---stdf64---stdstring)
    - [`to_string_precision : Std::U8 -> Std::F64 -> Std::String`](#to_string_precision--stdu8---stdf64---stdstring)
  - [`namespace Std::FFI`](#namespace-stdffi-1)
    - [`_unsafe_get_boxed_data_ptr : a -> Std::Ptr`](#_unsafe_get_boxed_data_ptr--a---stdptr)
    - [`unsafe_borrow_boxed_data_ptr : (Std::Ptr -> b) -> a -> b`](#unsafe_borrow_boxed_data_ptr--stdptr---b---a---b)
    - [`unsafe_clear_errno : () -> ()`](#unsafe_clear_errno-----)
    - [`unsafe_get_boxed_value_from_retained_ptr : Std::Ptr -> a`](#unsafe_get_boxed_value_from_retained_ptr--stdptr---a)
    - [`unsafe_get_errno : () -> Std::I32`](#unsafe_get_errno-----stdi32)
    - [`unsafe_get_release_function_of_boxed_value : (() -> a) -> Std::Ptr`](#unsafe_get_release_function_of_boxed_value-----a---stdptr)
    - [`unsafe_get_retain_function_of_boxed_value : (() -> a) -> Std::Ptr`](#unsafe_get_retain_function_of_boxed_value-----a---stdptr)
    - [`unsafe_get_retained_ptr_of_boxed_value : a -> Std::Ptr`](#unsafe_get_retained_ptr_of_boxed_value--a---stdptr)
  - [`namespace Std::FFI::Destructor`](#namespace-stdffidestructor)
    - [`@_value : Std::FFI::Destructor a -> a`](#_value--stdffidestructor-a---a)
    - [`@dtor : Std::FFI::Destructor a -> a -> ()`](#dtor--stdffidestructor-a---a---)
    - [`act__value : [f : Std::Functor] (a -> f a) -> Std::FFI::Destructor a -> f (Std::FFI::Destructor a)`](#act__value--f--stdfunctor-a---f-a---stdffidestructor-a---f-stdffidestructor-a)
    - [`act_dtor : [f : Std::Functor] ((a -> ()) -> f (a -> ())) -> Std::FFI::Destructor a -> f (Std::FFI::Destructor a)`](#act_dtor--f--stdfunctor-a------f-a------stdffidestructor-a---f-stdffidestructor-a)
    - [`borrow : (a -> b) -> Std::FFI::Destructor a -> b`](#borrow--a---b---stdffidestructor-a---b)
    - [`make : a -> (a -> ()) -> Std::FFI::Destructor a`](#make--a---a------stdffidestructor-a)
    - [`mod__value : (a -> a) -> Std::FFI::Destructor a -> Std::FFI::Destructor a`](#mod__value--a---a---stdffidestructor-a---stdffidestructor-a)
    - [`mod_dtor : ((a -> ()) -> a -> ()) -> Std::FFI::Destructor a -> Std::FFI::Destructor a`](#mod_dtor--a------a------stdffidestructor-a---stdffidestructor-a)
    - [`set__value : a -> Std::FFI::Destructor a -> Std::FFI::Destructor a`](#set__value--a---stdffidestructor-a---stdffidestructor-a)
    - [`set_dtor : (a -> ()) -> Std::FFI::Destructor a -> Std::FFI::Destructor a`](#set_dtor--a------stdffidestructor-a---stdffidestructor-a)
  - [`namespace Std::FromBytes`](#namespace-stdfrombytes)
    - [`from_bytes : [a : Std::FromBytes] Std::Array Std::U8 -> Std::Result Std::String a`](#from_bytes--a--stdfrombytes-stdarray-stdu8---stdresult-stdstring-a)
  - [`namespace Std::FromString`](#namespace-stdfromstring)
    - [`from_string : [a : Std::FromString] Std::String -> Std::Result Std::String a`](#from_string--a--stdfromstring-stdstring---stdresult-stdstring-a)
  - [`namespace Std::Functor`](#namespace-stdfunctor)
    - [`forget : [f : Std::Functor] f a -> f ()`](#forget--f--stdfunctor-f-a---f-)
    - [`map : [f : Std::Functor] (a -> b) -> f a -> f b`](#map--f--stdfunctor-a---b---f-a---f-b)
  - [`namespace Std::I16`](#namespace-stdi16)
    - [`abs : Std::I16 -> Std::I16`](#abs--stdi16---stdi16)
    - [`bit_and : Std::I16 -> Std::I16 -> Std::I16`](#bit_and--stdi16---stdi16---stdi16)
    - [`bit_or : Std::I16 -> Std::I16 -> Std::I16`](#bit_or--stdi16---stdi16---stdi16)
    - [`bit_xor : Std::I16 -> Std::I16 -> Std::I16`](#bit_xor--stdi16---stdi16---stdi16)
    - [`maximum : Std::I16`](#maximum--stdi16)
    - [`minimum : Std::I16`](#minimum--stdi16)
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
    - [`abs : Std::I32 -> Std::I32`](#abs--stdi32---stdi32)
    - [`bit_and : Std::I32 -> Std::I32 -> Std::I32`](#bit_and--stdi32---stdi32---stdi32)
    - [`bit_or : Std::I32 -> Std::I32 -> Std::I32`](#bit_or--stdi32---stdi32---stdi32)
    - [`bit_xor : Std::I32 -> Std::I32 -> Std::I32`](#bit_xor--stdi32---stdi32---stdi32)
    - [`maximum : Std::I32`](#maximum--stdi32)
    - [`minimum : Std::I32`](#minimum--stdi32)
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
    - [`abs : Std::I64 -> Std::I64`](#abs--stdi64---stdi64)
    - [`bit_and : Std::I64 -> Std::I64 -> Std::I64`](#bit_and--stdi64---stdi64---stdi64)
    - [`bit_or : Std::I64 -> Std::I64 -> Std::I64`](#bit_or--stdi64---stdi64---stdi64)
    - [`bit_xor : Std::I64 -> Std::I64 -> Std::I64`](#bit_xor--stdi64---stdi64---stdi64)
    - [`maximum : Std::I64`](#maximum--stdi64)
    - [`minimum : Std::I64`](#minimum--stdi64)
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
    - [`abs : Std::I8 -> Std::I8`](#abs--stdi8---stdi8)
    - [`bit_and : Std::I8 -> Std::I8 -> Std::I8`](#bit_and--stdi8---stdi8---stdi8)
    - [`bit_or : Std::I8 -> Std::I8 -> Std::I8`](#bit_or--stdi8---stdi8---stdi8)
    - [`bit_xor : Std::I8 -> Std::I8 -> Std::I8`](#bit_xor--stdi8---stdi8---stdi8)
    - [`maximum : Std::I8`](#maximum--stdi8)
    - [`minimum : Std::I8`](#minimum--stdi8)
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
    - [`@_data : Std::IO a -> () -> a`](#_data--stdio-a------a)
    - [`_read_line_inner : Std::Bool -> Std::IO::IOHandle -> Std::IO::IOFail Std::String`](#_read_line_inner--stdbool---stdioiohandle---stdioiofail-stdstring)
    - [`_unsafe_perform : Std::IO a -> a`](#_unsafe_perform--stdio-a---a)
    - [`act__data : [f : Std::Functor] ((() -> a) -> f (() -> a)) -> Std::IO a -> f (Std::IO a)`](#act__data--f--stdfunctor----a---f----a---stdio-a---f-stdio-a)
    - [`close_file : Std::IO::IOHandle -> Std::IO ()`](#close_file--stdioiohandle---stdio-)
    - [`eprint : Std::String -> Std::IO ()`](#eprint--stdstring---stdio-)
    - [`eprintln : Std::String -> Std::IO ()`](#eprintln--stdstring---stdio-)
    - [`exit : Std::I64 -> Std::IO a`](#exit--stdi64---stdio-a)
    - [`exit_with_msg : Std::I64 -> Std::String -> Std::IO a`](#exit_with_msg--stdi64---stdstring---stdio-a)
    - [`from_func : (() -> a) -> Std::IO a`](#from_func-----a---stdio-a)
    - [`get_arg : Std::I64 -> Std::IO (Std::Option Std::String)`](#get_arg--stdi64---stdio-stdoption-stdstring)
    - [`get_arg_count : Std::IO Std::I64`](#get_arg_count--stdio-stdi64)
    - [`get_args : Std::IO (Std::Array Std::String)`](#get_args--stdio-stdarray-stdstring)
    - [`input_line : Std::IO Std::String`](#input_line--stdio-stdstring)
    - [`is_eof : Std::IO::IOHandle -> Std::IO Std::Bool`](#is_eof--stdioiohandle---stdio-stdbool)
    - [`loop_lines : Std::IO::IOHandle -> s -> (s -> Std::String -> Std::LoopResult s s) -> Std::IO::IOFail s`](#loop_lines--stdioiohandle---s---s---stdstring---stdloopresult-s-s---stdioiofail-s)
    - [`loop_lines_io : Std::IO::IOHandle -> s -> (s -> Std::String -> Std::IO::IOFail (Std::LoopResult s s)) -> Std::IO::IOFail s`](#loop_lines_io--stdioiohandle---s---s---stdstring---stdioiofail-stdloopresult-s-s---stdioiofail-s)
    - [`mod__data : ((() -> a) -> () -> a) -> Std::IO a -> Std::IO a`](#mod__data-----a------a---stdio-a---stdio-a)
    - [`open_file : Std::Path -> Std::String -> Std::IO::IOFail Std::IO::IOHandle`](#open_file--stdpath---stdstring---stdioiofail-stdioiohandle)
    - [`print : Std::String -> Std::IO ()`](#print--stdstring---stdio-)
    - [`println : Std::String -> Std::IO ()`](#println--stdstring---stdio-)
    - [`read_bytes : Std::IO::IOHandle -> Std::IO::IOFail (Std::Array Std::U8)`](#read_bytes--stdioiohandle---stdioiofail-stdarray-stdu8)
    - [`read_file_bytes : Std::Path -> Std::IO::IOFail (Std::Array Std::U8)`](#read_file_bytes--stdpath---stdioiofail-stdarray-stdu8)
    - [`read_file_string : Std::Path -> Std::IO::IOFail Std::String`](#read_file_string--stdpath---stdioiofail-stdstring)
    - [`read_line : Std::IO::IOHandle -> Std::IO::IOFail Std::String`](#read_line--stdioiohandle---stdioiofail-stdstring)
    - [`read_n_bytes : Std::IO::IOHandle -> Std::I64 -> Std::IO::IOFail (Std::Array Std::U8)`](#read_n_bytes--stdioiohandle---stdi64---stdioiofail-stdarray-stdu8)
    - [`read_string : Std::IO::IOHandle -> Std::IO::IOFail Std::String`](#read_string--stdioiohandle---stdioiofail-stdstring)
    - [`set__data : (() -> a) -> Std::IO a -> Std::IO a`](#set__data-----a---stdio-a---stdio-a)
    - [`stderr : Std::IO::IOHandle`](#stderr--stdioiohandle)
    - [`stdin : Std::IO::IOHandle`](#stdin--stdioiohandle)
    - [`stdout : Std::IO::IOHandle`](#stdout--stdioiohandle)
    - [`with_file : Std::Path -> Std::String -> (Std::IO::IOHandle -> Std::IO::IOFail a) -> Std::IO::IOFail a`](#with_file--stdpath---stdstring---stdioiohandle---stdioiofail-a---stdioiofail-a)
    - [`write_bytes : Std::IO::IOHandle -> Std::Array Std::U8 -> Std::IO::IOFail ()`](#write_bytes--stdioiohandle---stdarray-stdu8---stdioiofail-)
    - [`write_file_bytes : Std::Path -> Std::Array Std::U8 -> Std::IO::IOFail ()`](#write_file_bytes--stdpath---stdarray-stdu8---stdioiofail-)
    - [`write_file_string : Std::Path -> Std::String -> Std::IO::IOFail ()`](#write_file_string--stdpath---stdstring---stdioiofail-)
    - [`write_string : Std::IO::IOHandle -> Std::String -> Std::IO::IOFail ()`](#write_string--stdioiohandle---stdstring---stdioiofail-)
  - [`namespace Std::IO::IOFail`](#namespace-stdioiofail)
    - [`@_data : Std::IO::IOFail a -> Std::IO (Std::Result Std::String a)`](#_data--stdioiofail-a---stdio-stdresult-stdstring-a)
    - [`act__data : [f : Std::Functor] (Std::IO (Std::Result Std::String a) -> f (Std::IO (Std::Result Std::String a))) -> Std::IO::IOFail a -> f (Std::IO::IOFail a)`](#act__data--f--stdfunctor-stdio-stdresult-stdstring-a---f-stdio-stdresult-stdstring-a---stdioiofail-a---f-stdioiofail-a)
    - [`from_result : Std::Result Std::String a -> Std::IO::IOFail a`](#from_result--stdresult-stdstring-a---stdioiofail-a)
    - [`lift : Std::IO a -> Std::IO::IOFail a`](#lift--stdio-a---stdioiofail-a)
    - [`mod__data : (Std::IO (Std::Result Std::String a) -> Std::IO (Std::Result Std::String a)) -> Std::IO::IOFail a -> Std::IO::IOFail a`](#mod__data--stdio-stdresult-stdstring-a---stdio-stdresult-stdstring-a---stdioiofail-a---stdioiofail-a)
    - [`set__data : Std::IO (Std::Result Std::String a) -> Std::IO::IOFail a -> Std::IO::IOFail a`](#set__data--stdio-stdresult-stdstring-a---stdioiofail-a---stdioiofail-a)
    - [`throw : Std::String -> Std::IO::IOFail a`](#throw--stdstring---stdioiofail-a)
    - [`to_result : Std::IO::IOFail a -> Std::IO (Std::Result Std::String a)`](#to_result--stdioiofail-a---stdio-stdresult-stdstring-a)
    - [`try : (Std::String -> Std::IO a) -> Std::IO::IOFail a -> Std::IO a`](#try--stdstring---stdio-a---stdioiofail-a---stdio-a)
  - [`namespace Std::IO::IOHandle`](#namespace-stdioiohandle)
    - [`@_data : Std::IO::IOHandle -> Std::FFI::Destructor Std::Ptr`](#_data--stdioiohandle---stdffidestructor-stdptr)
    - [`_file_ptr : Std::IO::IOHandle -> Std::Ptr`](#_file_ptr--stdioiohandle---stdptr)
    - [`_unsafe_close : Std::IO::IOHandle -> ()`](#_unsafe_close--stdioiohandle---)
    - [`act__data : [f : Std::Functor] (Std::FFI::Destructor Std::Ptr -> f (Std::FFI::Destructor Std::Ptr)) -> Std::IO::IOHandle -> f Std::IO::IOHandle`](#act__data--f--stdfunctor-stdffidestructor-stdptr---f-stdffidestructor-stdptr---stdioiohandle---f-stdioiohandle)
    - [`from_file_ptr : Std::Ptr -> Std::IO::IOHandle`](#from_file_ptr--stdptr---stdioiohandle)
    - [`mod__data : (Std::FFI::Destructor Std::Ptr -> Std::FFI::Destructor Std::Ptr) -> Std::IO::IOHandle -> Std::IO::IOHandle`](#mod__data--stdffidestructor-stdptr---stdffidestructor-stdptr---stdioiohandle---stdioiohandle)
    - [`set__data : Std::FFI::Destructor Std::Ptr -> Std::IO::IOHandle -> Std::IO::IOHandle`](#set__data--stdffidestructor-stdptr---stdioiohandle---stdioiohandle)
  - [`namespace Std::Iterator`](#namespace-stditerator)
    - [`@next : Std::Iterator a -> () -> Std::Option (a, Std::Iterator a)`](#next--stditerator-a------stdoption-a-stditerator-a)
    - [`_flatten : Std::Iterator (Std::Iterator a) -> Std::Iterator a`](#_flatten--stditerator-stditerator-a---stditerator-a)
    - [`_flatten_sub : Std::Iterator a -> Std::Iterator (Std::Iterator a) -> Std::Iterator a`](#_flatten_sub--stditerator-a---stditerator-stditerator-a---stditerator-a)
    - [`act_next : [f : Std::Functor] ((() -> Std::Option (a, Std::Iterator a)) -> f (() -> Std::Option (a, Std::Iterator a))) -> Std::Iterator a -> f (Std::Iterator a)`](#act_next--f--stdfunctor----stdoption-a-stditerator-a---f----stdoption-a-stditerator-a---stditerator-a---f-stditerator-a)
    - [`advance : Std::Iterator a -> Std::Option (a, Std::Iterator a)`](#advance--stditerator-a---stdoption-a-stditerator-a)
    - [`append : Std::Iterator a -> Std::Iterator a -> Std::Iterator a`](#append--stditerator-a---stditerator-a---stditerator-a)
    - [`bang : Std::Iterator a -> Std::Iterator a`](#bang--stditerator-a---stditerator-a)
    - [`count_up : Std::I64 -> Std::Iterator Std::I64`](#count_up--stdi64---stditerator-stdi64)
    - [`empty : Std::Iterator a`](#empty--stditerator-a)
    - [`filter : (a -> Std::Bool) -> Std::Iterator a -> Std::Iterator a`](#filter--a---stdbool---stditerator-a---stditerator-a)
    - [`find_last : Std::Iterator a -> Std::Option a`](#find_last--stditerator-a---stdoption-a)
    - [`fold : b -> (b -> a -> b) -> Std::Iterator a -> b`](#fold--b---b---a---b---stditerator-a---b)
    - [`fold_m : [m : Std::Monad] b -> (b -> a -> m b) -> Std::Iterator a -> m b`](#fold_m--m--stdmonad-b---b---a---m-b---stditerator-a---m-b)
    - [`from_array : Std::Array a -> Std::Iterator a`](#from_array--stdarray-a---stditerator-a)
    - [`from_map : (Std::I64 -> a) -> Std::Iterator a`](#from_map--stdi64---a---stditerator-a)
    - [`generate : s -> (s -> Std::Option (a, s)) -> Std::Iterator a`](#generate--s---s---stdoption-a-s---stditerator-a)
    - [`get_first : Std::Iterator a -> Std::Option a`](#get_first--stditerator-a---stdoption-a)
    - [`get_size : Std::Iterator a -> Std::I64`](#get_size--stditerator-a---stdi64)
    - [`get_tail : Std::Iterator a -> Std::Option (Std::Iterator a)`](#get_tail--stditerator-a---stdoption-stditerator-a)
    - [`intersperse : a -> Std::Iterator a -> Std::Iterator a`](#intersperse--a---stditerator-a---stditerator-a)
    - [`is_empty : Std::Iterator a -> Std::Bool`](#is_empty--stditerator-a---stdbool)
    - [`loop_iter : b -> (b -> a -> Std::LoopResult b b) -> Std::Iterator a -> b`](#loop_iter--b---b---a---stdloopresult-b-b---stditerator-a---b)
    - [`loop_iter_m : [m : Std::Monad] b -> (b -> a -> m (Std::LoopResult b b)) -> Std::Iterator a -> m b`](#loop_iter_m--m--stdmonad-b---b---a---m-stdloopresult-b-b---stditerator-a---m-b)
    - [`mod_next : ((() -> Std::Option (a, Std::Iterator a)) -> () -> Std::Option (a, Std::Iterator a)) -> Std::Iterator a -> Std::Iterator a`](#mod_next-----stdoption-a-stditerator-a------stdoption-a-stditerator-a---stditerator-a---stditerator-a)
    - [`product : Std::Iterator a -> Std::Iterator b -> Std::Iterator (b, a)`](#product--stditerator-a---stditerator-b---stditerator-b-a)
    - [`push_front : a -> Std::Iterator a -> Std::Iterator a`](#push_front--a---stditerator-a---stditerator-a)
    - [`range : Std::I64 -> Std::I64 -> Std::Iterator Std::I64`](#range--stdi64---stdi64---stditerator-stdi64)
    - [`reverse : Std::Iterator a -> Std::Iterator a`](#reverse--stditerator-a---stditerator-a)
    - [`set_next : (() -> Std::Option (a, Std::Iterator a)) -> Std::Iterator a -> Std::Iterator a`](#set_next-----stdoption-a-stditerator-a---stditerator-a---stditerator-a)
    - [`subsequences : Std::Iterator a -> Std::Iterator (Std::Iterator a)`](#subsequences--stditerator-a---stditerator-stditerator-a)
    - [`sum : [a : Std::Additive] Std::Iterator a -> a`](#sum--a--stdadditive-stditerator-a---a)
    - [`take : Std::I64 -> Std::Iterator a -> Std::Iterator a`](#take--stdi64---stditerator-a---stditerator-a)
    - [`take_while : (a -> Std::Bool) -> Std::Iterator a -> Std::Iterator a`](#take_while--a---stdbool---stditerator-a---stditerator-a)
    - [`to_array : Std::Iterator a -> Std::Array a`](#to_array--stditerator-a---stdarray-a)
    - [`zip : Std::Iterator b -> Std::Iterator a -> Std::Iterator (a, b)`](#zip--stditerator-b---stditerator-a---stditerator-a-b)
  - [`namespace Std::LessThan`](#namespace-stdlessthan)
    - [`less_than : [a : Std::LessThan] a -> a -> Std::Bool`](#less_than--a--stdlessthan-a---a---stdbool)
    - [`max : [a : Std::LessThan] a -> a -> a`](#max--a--stdlessthan-a---a---a)
    - [`min : [a : Std::LessThan] a -> a -> a`](#min--a--stdlessthan-a---a---a)
  - [`namespace Std::LessThanOrEq`](#namespace-stdlessthanoreq)
    - [`less_than_or_eq : [a : Std::LessThanOrEq] a -> a -> Std::Bool`](#less_than_or_eq--a--stdlessthanoreq-a---a---stdbool)
  - [`namespace Std::LoopResult`](#namespace-stdloopresult)
    - [`as_break : Std::LoopResult s b -> b`](#as_break--stdloopresult-s-b---b)
    - [`as_continue : Std::LoopResult s b -> s`](#as_continue--stdloopresult-s-b---s)
    - [`break : b -> Std::LoopResult s b`](#break--b---stdloopresult-s-b)
    - [`break_m : [m : Std::Monad] r -> m (Std::LoopResult s r)`](#break_m--m--stdmonad-r---m-stdloopresult-s-r)
    - [`continue : s -> Std::LoopResult s b`](#continue--s---stdloopresult-s-b)
    - [`continue_m : [m : Std::Monad] s -> m (Std::LoopResult s r)`](#continue_m--m--stdmonad-s---m-stdloopresult-s-r)
    - [`is_break : Std::LoopResult s b -> Std::Bool`](#is_break--stdloopresult-s-b---stdbool)
    - [`is_continue : Std::LoopResult s b -> Std::Bool`](#is_continue--stdloopresult-s-b---stdbool)
    - [`mod_break : (b -> b) -> Std::LoopResult s b -> Std::LoopResult s b`](#mod_break--b---b---stdloopresult-s-b---stdloopresult-s-b)
    - [`mod_continue : (s -> s) -> Std::LoopResult s b -> Std::LoopResult s b`](#mod_continue--s---s---stdloopresult-s-b---stdloopresult-s-b)
  - [`namespace Std::Monad`](#namespace-stdmonad)
    - [`bind : [m : Std::Monad] (a -> m b) -> m a -> m b`](#bind--m--stdmonad-a---m-b---m-a---m-b)
    - [`flatten : [m : Std::Monad] m (m a) -> m a`](#flatten--m--stdmonad-m-m-a---m-a)
    - [`pure : [m : Std::Monad] a -> m a`](#pure--m--stdmonad-a---m-a)
    - [`unless : [m : Std::Monad] Std::Bool -> m () -> m ()`](#unless--m--stdmonad-stdbool---m----m-)
    - [`when : [m : Std::Monad] Std::Bool -> m () -> m ()`](#when--m--stdmonad-stdbool---m----m-)
  - [`namespace Std::Mul`](#namespace-stdmul)
    - [`mul : [a : Std::Mul] a -> a -> a`](#mul--a--stdmul-a---a---a)
  - [`namespace Std::Neg`](#namespace-stdneg)
    - [`neg : [a : Std::Neg] a -> a`](#neg--a--stdneg-a---a)
  - [`namespace Std::Not`](#namespace-stdnot)
    - [`not : [a : Std::Not] a -> a`](#not--a--stdnot-a---a)
  - [`namespace Std::Option`](#namespace-stdoption)
    - [`as_none : Std::Option a -> ()`](#as_none--stdoption-a---)
    - [`as_some : Std::Option a -> a`](#as_some--stdoption-a---a)
    - [`as_some_or : a -> Std::Option a -> a`](#as_some_or--a---stdoption-a---a)
    - [`is_none : Std::Option a -> Std::Bool`](#is_none--stdoption-a---stdbool)
    - [`is_some : Std::Option a -> Std::Bool`](#is_some--stdoption-a---stdbool)
    - [`map_or : b -> (a -> b) -> Std::Option a -> b`](#map_or--b---a---b---stdoption-a---b)
    - [`mod_none : (() -> ()) -> Std::Option a -> Std::Option a`](#mod_none--------stdoption-a---stdoption-a)
    - [`mod_some : (a -> a) -> Std::Option a -> Std::Option a`](#mod_some--a---a---stdoption-a---stdoption-a)
    - [`none : () -> Std::Option a`](#none-----stdoption-a)
    - [`some : a -> Std::Option a`](#some--a---stdoption-a)
  - [`namespace Std::Path`](#namespace-stdpath)
    - [`@_data : Std::Path -> Std::String`](#_data--stdpath---stdstring)
    - [`act__data : [f : Std::Functor] (Std::String -> f Std::String) -> Std::Path -> f Std::Path`](#act__data--f--stdfunctor-stdstring---f-stdstring---stdpath---f-stdpath)
    - [`mod__data : (Std::String -> Std::String) -> Std::Path -> Std::Path`](#mod__data--stdstring---stdstring---stdpath---stdpath)
    - [`parse : Std::String -> Std::Option Std::Path`](#parse--stdstring---stdoption-stdpath)
    - [`set__data : Std::String -> Std::Path -> Std::Path`](#set__data--stdstring---stdpath---stdpath)
  - [`namespace Std::Ptr`](#namespace-stdptr)
    - [`add_offset : Std::I64 -> Std::Ptr -> Std::Ptr`](#add_offset--stdi64---stdptr---stdptr)
    - [`subtract_ptr : Std::Ptr -> Std::Ptr -> Std::I64`](#subtract_ptr--stdptr---stdptr---stdi64)
  - [`namespace Std::PunchedArray`](#namespace-stdpunchedarray)
    - [`@_data : Std::PunchedArray a -> Std::FFI::Destructor (Std::Array a)`](#_data--stdpunchedarray-a---stdffidestructor-stdarray-a)
    - [`@idx : Std::PunchedArray a -> Std::I64`](#idx--stdpunchedarray-a---stdi64)
    - [`act__data : [f : Std::Functor] (Std::FFI::Destructor (Std::Array a) -> f (Std::FFI::Destructor (Std::Array a))) -> Std::PunchedArray a -> f (Std::PunchedArray a)`](#act__data--f--stdfunctor-stdffidestructor-stdarray-a---f-stdffidestructor-stdarray-a---stdpunchedarray-a---f-stdpunchedarray-a)
    - [`act_idx : [f : Std::Functor] (Std::I64 -> f Std::I64) -> Std::PunchedArray a -> f (Std::PunchedArray a)`](#act_idx--f--stdfunctor-stdi64---f-stdi64---stdpunchedarray-a---f-stdpunchedarray-a)
    - [`mod__data : (Std::FFI::Destructor (Std::Array a) -> Std::FFI::Destructor (Std::Array a)) -> Std::PunchedArray a -> Std::PunchedArray a`](#mod__data--stdffidestructor-stdarray-a---stdffidestructor-stdarray-a---stdpunchedarray-a---stdpunchedarray-a)
    - [`mod_idx : (Std::I64 -> Std::I64) -> Std::PunchedArray a -> Std::PunchedArray a`](#mod_idx--stdi64---stdi64---stdpunchedarray-a---stdpunchedarray-a)
    - [`plug_in : a -> Std::PunchedArray a -> Std::Array a`](#plug_in--a---stdpunchedarray-a---stdarray-a)
    - [`set__data : Std::FFI::Destructor (Std::Array a) -> Std::PunchedArray a -> Std::PunchedArray a`](#set__data--stdffidestructor-stdarray-a---stdpunchedarray-a---stdpunchedarray-a)
    - [`set_idx : Std::I64 -> Std::PunchedArray a -> Std::PunchedArray a`](#set_idx--stdi64---stdpunchedarray-a---stdpunchedarray-a)
    - [`unsafe_punch : Std::I64 -> Std::Array a -> (Std::PunchedArray a, a)`](#unsafe_punch--stdi64---stdarray-a---stdpunchedarray-a-a)
  - [`namespace Std::Rem`](#namespace-stdrem)
    - [`rem : [a : Std::Rem] a -> a -> a`](#rem--a--stdrem-a---a---a)
  - [`namespace Std::Result`](#namespace-stdresult)
    - [`as_err : Std::Result e o -> e`](#as_err--stdresult-e-o---e)
    - [`as_ok : Std::Result e o -> o`](#as_ok--stdresult-e-o---o)
    - [`err : e -> Std::Result e o`](#err--e---stdresult-e-o)
    - [`is_err : Std::Result e o -> Std::Bool`](#is_err--stdresult-e-o---stdbool)
    - [`is_ok : Std::Result e o -> Std::Bool`](#is_ok--stdresult-e-o---stdbool)
    - [`mod_err : (e -> e) -> Std::Result e o -> Std::Result e o`](#mod_err--e---e---stdresult-e-o---stdresult-e-o)
    - [`mod_ok : (o -> o) -> Std::Result e o -> Std::Result e o`](#mod_ok--o---o---stdresult-e-o---stdresult-e-o)
    - [`ok : o -> Std::Result e o`](#ok--o---stdresult-e-o)
    - [`unwrap : Std::Result e o -> o`](#unwrap--stdresult-e-o---o)
  - [`namespace Std::String`](#namespace-stdstring)
    - [`@_data : Std::String -> Std::Array Std::U8`](#_data--stdstring---stdarray-stdu8)
    - [`_get_c_str : Std::String -> Std::Ptr`](#_get_c_str--stdstring---stdptr)
    - [`_unsafe_from_c_str : Std::Array Std::U8 -> Std::String`](#_unsafe_from_c_str--stdarray-stdu8---stdstring)
    - [`_unsafe_from_c_str_ptr : Std::Ptr -> Std::String`](#_unsafe_from_c_str_ptr--stdptr---stdstring)
    - [`act__data : [f : Std::Functor] (Std::Array Std::U8 -> f (Std::Array Std::U8)) -> Std::String -> f Std::String`](#act__data--f--stdfunctor-stdarray-stdu8---f-stdarray-stdu8---stdstring---f-stdstring)
    - [`borrow_c_str : (Std::Ptr -> a) -> Std::String -> a`](#borrow_c_str--stdptr---a---stdstring---a)
    - [`concat : Std::String -> Std::String -> Std::String`](#concat--stdstring---stdstring---stdstring)
    - [`concat_iter : Std::Iterator Std::String -> Std::String`](#concat_iter--stditerator-stdstring---stdstring)
    - [`empty : Std::I64 -> Std::String`](#empty--stdi64---stdstring)
    - [`find : Std::String -> Std::I64 -> Std::String -> Std::Option Std::I64`](#find--stdstring---stdi64---stdstring---stdoption-stdi64)
    - [`get_bytes : Std::String -> Std::Array Std::U8`](#get_bytes--stdstring---stdarray-stdu8)
    - [`get_first_byte : Std::String -> Std::Option Std::U8`](#get_first_byte--stdstring---stdoption-stdu8)
    - [`get_last_byte : Std::String -> Std::Option Std::U8`](#get_last_byte--stdstring---stdoption-stdu8)
    - [`get_size : Std::String -> Std::I64`](#get_size--stdstring---stdi64)
    - [`get_sub : Std::I64 -> Std::I64 -> Std::String -> Std::String`](#get_sub--stdi64---stdi64---stdstring---stdstring)
    - [`is_empty : Std::String -> Std::Bool`](#is_empty--stdstring---stdbool)
    - [`join : Std::String -> Std::Iterator Std::String -> Std::String`](#join--stdstring---stditerator-stdstring---stdstring)
    - [`mod__data : (Std::Array Std::U8 -> Std::Array Std::U8) -> Std::String -> Std::String`](#mod__data--stdarray-stdu8---stdarray-stdu8---stdstring---stdstring)
    - [`pop_back_byte : Std::String -> Std::String`](#pop_back_byte--stdstring---stdstring)
    - [`set__data : Std::Array Std::U8 -> Std::String -> Std::String`](#set__data--stdarray-stdu8---stdstring---stdstring)
    - [`split : Std::String -> Std::String -> Std::Iterator Std::String`](#split--stdstring---stdstring---stditerator-stdstring)
    - [`strip_first_bytes : (Std::U8 -> Std::Bool) -> Std::String -> Std::String`](#strip_first_bytes--stdu8---stdbool---stdstring---stdstring)
    - [`strip_first_spaces : Std::String -> Std::String`](#strip_first_spaces--stdstring---stdstring)
    - [`strip_last_bytes : (Std::U8 -> Std::Bool) -> Std::String -> Std::String`](#strip_last_bytes--stdu8---stdbool---stdstring---stdstring)
    - [`strip_last_newlines : Std::String -> Std::String`](#strip_last_newlines--stdstring---stdstring)
    - [`strip_last_spaces : Std::String -> Std::String`](#strip_last_spaces--stdstring---stdstring)
    - [`strip_spaces : Std::String -> Std::String`](#strip_spaces--stdstring---stdstring)
  - [`namespace Std::Sub`](#namespace-stdsub)
    - [`sub : [a : Std::Sub] a -> a -> a`](#sub--a--stdsub-a---a---a)
  - [`namespace Std::ToBytes`](#namespace-stdtobytes)
    - [`to_bytes : [a : Std::ToBytes] a -> Std::Array Std::U8`](#to_bytes--a--stdtobytes-a---stdarray-stdu8)
  - [`namespace Std::ToString`](#namespace-stdtostring)
    - [`to_string : [a : Std::ToString] a -> Std::String`](#to_string--a--stdtostring-a---stdstring)
  - [`namespace Std::Tuple2`](#namespace-stdtuple2)
    - [`@0 : (t0, t1) -> t0`](#0--t0-t1---t0)
    - [`@1 : (t0, t1) -> t1`](#1--t0-t1---t1)
    - [`act_0 : [f : Std::Functor] (t0 -> f t0) -> (t0, t1) -> f (t0, t1)`](#act_0--f--stdfunctor-t0---f-t0---t0-t1---f-t0-t1)
    - [`act_1 : [f : Std::Functor] (t1 -> f t1) -> (t0, t1) -> f (t0, t1)`](#act_1--f--stdfunctor-t1---f-t1---t0-t1---f-t0-t1)
    - [`mod_0 : (t0 -> t0) -> (t0, t1) -> (t0, t1)`](#mod_0--t0---t0---t0-t1---t0-t1)
    - [`mod_1 : (t1 -> t1) -> (t0, t1) -> (t0, t1)`](#mod_1--t1---t1---t0-t1---t0-t1)
    - [`set_0 : t0 -> (t0, t1) -> (t0, t1)`](#set_0--t0---t0-t1---t0-t1)
    - [`set_1 : t1 -> (t0, t1) -> (t0, t1)`](#set_1--t1---t0-t1---t0-t1)
  - [`namespace Std::Tuple3`](#namespace-stdtuple3)
    - [`@0 : (t0, t1, t2) -> t0`](#0--t0-t1-t2---t0)
    - [`@1 : (t0, t1, t2) -> t1`](#1--t0-t1-t2---t1)
    - [`@2 : (t0, t1, t2) -> t2`](#2--t0-t1-t2---t2)
    - [`act_0 : [f : Std::Functor] (t0 -> f t0) -> (t0, t1, t2) -> f (t0, t1, t2)`](#act_0--f--stdfunctor-t0---f-t0---t0-t1-t2---f-t0-t1-t2)
    - [`act_1 : [f : Std::Functor] (t1 -> f t1) -> (t0, t1, t2) -> f (t0, t1, t2)`](#act_1--f--stdfunctor-t1---f-t1---t0-t1-t2---f-t0-t1-t2)
    - [`act_2 : [f : Std::Functor] (t2 -> f t2) -> (t0, t1, t2) -> f (t0, t1, t2)`](#act_2--f--stdfunctor-t2---f-t2---t0-t1-t2---f-t0-t1-t2)
    - [`mod_0 : (t0 -> t0) -> (t0, t1, t2) -> (t0, t1, t2)`](#mod_0--t0---t0---t0-t1-t2---t0-t1-t2)
    - [`mod_1 : (t1 -> t1) -> (t0, t1, t2) -> (t0, t1, t2)`](#mod_1--t1---t1---t0-t1-t2---t0-t1-t2)
    - [`mod_2 : (t2 -> t2) -> (t0, t1, t2) -> (t0, t1, t2)`](#mod_2--t2---t2---t0-t1-t2---t0-t1-t2)
    - [`set_0 : t0 -> (t0, t1, t2) -> (t0, t1, t2)`](#set_0--t0---t0-t1-t2---t0-t1-t2)
    - [`set_1 : t1 -> (t0, t1, t2) -> (t0, t1, t2)`](#set_1--t1---t0-t1-t2---t0-t1-t2)
    - [`set_2 : t2 -> (t0, t1, t2) -> (t0, t1, t2)`](#set_2--t2---t0-t1-t2---t0-t1-t2)
  - [`namespace Std::U16`](#namespace-stdu16)
    - [`bit_and : Std::U16 -> Std::U16 -> Std::U16`](#bit_and--stdu16---stdu16---stdu16)
    - [`bit_or : Std::U16 -> Std::U16 -> Std::U16`](#bit_or--stdu16---stdu16---stdu16)
    - [`bit_xor : Std::U16 -> Std::U16 -> Std::U16`](#bit_xor--stdu16---stdu16---stdu16)
    - [`maximum : Std::U16`](#maximum--stdu16)
    - [`minimum : Std::U16`](#minimum--stdu16)
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
    - [`maximum : Std::U32`](#maximum--stdu32)
    - [`minimum : Std::U32`](#minimum--stdu32)
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
    - [`maximum : Std::U64`](#maximum--stdu64)
    - [`minimum : Std::U64`](#minimum--stdu64)
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
    - [`maximum : Std::U8`](#maximum--stdu8)
    - [`minimum : Std::U8`](#minimum--stdu8)
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
  - [`namespace Std::Zero`](#namespace-stdzero)
    - [`zero : [a : Std::Zero] a`](#zero--a--stdzero-a)


# `module Std`

Module `Std` provides basic types, traits and values.

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

### `type ErrMsg = Std::String`

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

#### field `next : () -> Std::Option (a, Std::Iterator a)`

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

#### field `_data : Std::String`

### `type Ptr = unbox { primitive }`

### `type PunchedArray a = unbox struct { ...fields... }`

The type of punched arrays.

A punched array is an array from which a certain element has been removed.
This is used in the implementation of `Array::act`.

#### field `_data : Std::FFI::Destructor (Std::Array a)`

#### field `idx : Std::I64`

### `type Result e o = unbox union { ...variants... }`

A type of result value for a computation that may fail.

#### variant `ok : o`

#### variant `err : e`

### `type String = unbox struct { ...fields... }`

#### field `_data : Std::Array Std::U8`

### `type Tuple0 = unbox struct { ...fields... }`

### `type Tuple2 t0 t1 = unbox struct { ...fields... }`

#### field `0 : t0`

#### field `1 : t1`

### `type Tuple3 t0 t1 t2 = unbox struct { ...fields... }`

#### field `0 : t0`

#### field `1 : t1`

#### field `2 : t2`

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

#### field `_data : Std::IO (Std::Result Std::String a)`

### `type IOHandle = unbox struct { ...fields... }`

A handle type for read / write operations on files, stdin, stdout, stderr.

You can create `IOHandle` value by `IO::open_file`, and close it by `IO::close_file`.

There are also global `IO::IOHandle::stdin`, `IO::IOHandle::stdout`, `IO::IOHandle::stderr`.

#### field `_data : Std::FFI::Destructor Std::Ptr`

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

#### method `eq : a -> a -> Std::Bool`

Checks equality of two values. An expression `x == y` is translated to `eq(x, y)`.

### `trait a : FromBytes`

#### method `from_bytes : Std::Array Std::U8 -> Std::Result Std::String a`

### `trait a : FromString`

#### method `from_string : Std::String -> Std::Result Std::String a`

### `trait [f : *->*] f : Functor`

#### method `map : (a -> b) -> f a -> f b`

### `trait a : LessThan`

Trait for infix operator `<`.

#### method `less_than : a -> a -> Std::Bool`

Compares two values. An expression `x < y` is translated to `less_than(x, y)`.

### `trait a : LessThanOrEq`

Trait for infix operator `<=`.

#### method `less_than_or_eq : a -> a -> Std::Bool`

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

#### method `to_bytes : a -> Std::Array Std::U8`

### `trait a : ToString`

#### method `to_string : a -> Std::String`

### `trait a : Zero`

#### method `zero : a`

# Trait implementations

### `impl () : Std::Eq`

### `impl () : Std::ToString`

Returns "()".

### `impl (t0, *) : Std::Functor`

### `impl [t0 : Std::Eq, t1 : Std::Eq] (t0, t1) : Std::Eq`

### `impl [t0 : Std::Eq, t0 : Std::LessThan, t1 : Std::Eq, t1 : Std::LessThan] (t0, t1) : Std::LessThan`

### `impl [t0 : Std::Eq, t0 : Std::LessThanOrEq, t1 : Std::Eq, t1 : Std::LessThanOrEq] (t0, t1) : Std::LessThanOrEq`

### `impl [t0 : Std::ToString, t1 : Std::ToString] (t0, t1) : Std::ToString`

### `impl (t0, t1, *) : Std::Functor`

### `impl [t0 : Std::Eq, t1 : Std::Eq, t2 : Std::Eq] (t0, t1, t2) : Std::Eq`

### `impl [t0 : Std::Eq, t0 : Std::LessThan, t1 : Std::Eq, t1 : Std::LessThan, t2 : Std::Eq, t2 : Std::LessThan] (t0, t1, t2) : Std::LessThan`

### `impl [t0 : Std::Eq, t0 : Std::LessThanOrEq, t1 : Std::Eq, t1 : Std::LessThanOrEq, t2 : Std::Eq, t2 : Std::LessThanOrEq] (t0, t1, t2) : Std::LessThanOrEq`

### `impl [t0 : Std::ToString, t1 : Std::ToString, t2 : Std::ToString] (t0, t1, t2) : Std::ToString`

### `impl Std::Array : Std::Functor`

### `impl Std::Array : Std::Monad`

### `impl [a : Std::Eq] Std::Array a : Std::Eq`

### `impl [a : Std::Eq, a : Std::LessThan] Std::Array a : Std::LessThan`

`LessThan` implementation for `Array a`.

Compares two arrays by lexicographic order.

### `impl [a : Std::Eq, a : Std::LessThanOrEq] Std::Array a : Std::LessThanOrEq`

`LessThanOrEq` implementation for `Array a`.

Compares two arrays by lexicographic order.

### `impl [a : Std::ToString] Std::Array a : Std::ToString`

### `impl Std::Bool : Std::Eq`

### `impl Std::Bool : Std::Not`

### `impl Std::Bool : Std::ToString`

### `impl Std::F32 : Std::Add`

### `impl Std::F32 : Std::Div`

### `impl Std::F32 : Std::Eq`

### `impl Std::F32 : Std::FromBytes`

### `impl Std::F32 : Std::FromString`

### `impl Std::F32 : Std::LessThan`

### `impl Std::F32 : Std::LessThanOrEq`

### `impl Std::F32 : Std::Mul`

### `impl Std::F32 : Std::Neg`

### `impl Std::F32 : Std::Sub`

### `impl Std::F32 : Std::ToBytes`

### `impl Std::F32 : Std::ToString`

### `impl Std::F32 : Std::Zero`

### `impl Std::F64 : Std::Add`

### `impl Std::F64 : Std::Div`

### `impl Std::F64 : Std::Eq`

### `impl Std::F64 : Std::FromBytes`

### `impl Std::F64 : Std::FromString`

### `impl Std::F64 : Std::LessThan`

### `impl Std::F64 : Std::LessThanOrEq`

### `impl Std::F64 : Std::Mul`

### `impl Std::F64 : Std::Neg`

### `impl Std::F64 : Std::Sub`

### `impl Std::F64 : Std::ToBytes`

### `impl Std::F64 : Std::ToString`

### `impl Std::F64 : Std::Zero`

### `impl Std::I16 : Std::Add`

### `impl Std::I16 : Std::Div`

### `impl Std::I16 : Std::Eq`

### `impl Std::I16 : Std::FromBytes`

### `impl Std::I16 : Std::FromString`

### `impl Std::I16 : Std::LessThan`

### `impl Std::I16 : Std::LessThanOrEq`

### `impl Std::I16 : Std::Mul`

### `impl Std::I16 : Std::Neg`

### `impl Std::I16 : Std::Rem`

### `impl Std::I16 : Std::Sub`

### `impl Std::I16 : Std::ToBytes`

### `impl Std::I16 : Std::ToString`

### `impl Std::I16 : Std::Zero`

### `impl Std::I32 : Std::Add`

### `impl Std::I32 : Std::Div`

### `impl Std::I32 : Std::Eq`

### `impl Std::I32 : Std::FromBytes`

### `impl Std::I32 : Std::FromString`

### `impl Std::I32 : Std::LessThan`

### `impl Std::I32 : Std::LessThanOrEq`

### `impl Std::I32 : Std::Mul`

### `impl Std::I32 : Std::Neg`

### `impl Std::I32 : Std::Rem`

### `impl Std::I32 : Std::Sub`

### `impl Std::I32 : Std::ToBytes`

### `impl Std::I32 : Std::ToString`

### `impl Std::I32 : Std::Zero`

### `impl Std::I64 : Std::Add`

### `impl Std::I64 : Std::Div`

### `impl Std::I64 : Std::Eq`

### `impl Std::I64 : Std::FromBytes`

### `impl Std::I64 : Std::FromString`

### `impl Std::I64 : Std::LessThan`

### `impl Std::I64 : Std::LessThanOrEq`

### `impl Std::I64 : Std::Mul`

### `impl Std::I64 : Std::Neg`

### `impl Std::I64 : Std::Rem`

### `impl Std::I64 : Std::Sub`

### `impl Std::I64 : Std::ToBytes`

### `impl Std::I64 : Std::ToString`

### `impl Std::I64 : Std::Zero`

### `impl Std::I8 : Std::Add`

### `impl Std::I8 : Std::Div`

### `impl Std::I8 : Std::Eq`

### `impl Std::I8 : Std::FromBytes`

### `impl Std::I8 : Std::FromString`

### `impl Std::I8 : Std::LessThan`

### `impl Std::I8 : Std::LessThanOrEq`

### `impl Std::I8 : Std::Mul`

### `impl Std::I8 : Std::Neg`

### `impl Std::I8 : Std::Rem`

### `impl Std::I8 : Std::Sub`

### `impl Std::I8 : Std::ToBytes`

### `impl Std::I8 : Std::ToString`

### `impl Std::I8 : Std::Zero`

### `impl Std::IO : Std::Functor`

### `impl Std::IO : Std::Monad`

### `impl Std::IO::IOFail : Std::Functor`

### `impl Std::IO::IOFail : Std::Monad`

### `impl Std::Iterator : Std::Functor`

### `impl Std::Iterator : Std::Monad`

### `impl Std::Iterator a : Std::Add`

### `impl [a : Std::Eq] Std::Iterator a : Std::Eq`

### `impl Std::Option : Std::Functor`

### `impl Std::Option : Std::Monad`

### `impl [a : Std::Eq] Std::Option a : Std::Eq`

### `impl [a : Std::ToString] Std::Option a : Std::ToString`

### `impl Std::Path : Std::ToString`

### `impl Std::Ptr : Std::Eq`

### `impl Std::Ptr : Std::ToString`

### `impl Std::Result e : Std::Functor`

### `impl Std::Result e : Std::Monad`

### `impl [e : Std::Eq, a : Std::Eq] Std::Result e a : Std::Eq`

### `impl [e : Std::ToString, a : Std::ToString] Std::Result e a : Std::ToString`

### `impl Std::String : Std::Add`

Concatenates two strings.

### `impl Std::String : Std::Eq`

### `impl Std::String : Std::LessThan`

### `impl Std::String : Std::LessThanOrEq`

### `impl Std::String : Std::ToString`

### `impl Std::U16 : Std::Add`

### `impl Std::U16 : Std::Div`

### `impl Std::U16 : Std::Eq`

### `impl Std::U16 : Std::FromBytes`

### `impl Std::U16 : Std::FromString`

### `impl Std::U16 : Std::LessThan`

### `impl Std::U16 : Std::LessThanOrEq`

### `impl Std::U16 : Std::Mul`

### `impl Std::U16 : Std::Neg`

### `impl Std::U16 : Std::Rem`

### `impl Std::U16 : Std::Sub`

### `impl Std::U16 : Std::ToBytes`

### `impl Std::U16 : Std::ToString`

### `impl Std::U16 : Std::Zero`

### `impl Std::U32 : Std::Add`

### `impl Std::U32 : Std::Div`

### `impl Std::U32 : Std::Eq`

### `impl Std::U32 : Std::FromBytes`

### `impl Std::U32 : Std::FromString`

### `impl Std::U32 : Std::LessThan`

### `impl Std::U32 : Std::LessThanOrEq`

### `impl Std::U32 : Std::Mul`

### `impl Std::U32 : Std::Neg`

### `impl Std::U32 : Std::Rem`

### `impl Std::U32 : Std::Sub`

### `impl Std::U32 : Std::ToBytes`

### `impl Std::U32 : Std::ToString`

### `impl Std::U32 : Std::Zero`

### `impl Std::U64 : Std::Add`

### `impl Std::U64 : Std::Div`

### `impl Std::U64 : Std::Eq`

### `impl Std::U64 : Std::FromBytes`

### `impl Std::U64 : Std::FromString`

### `impl Std::U64 : Std::LessThan`

### `impl Std::U64 : Std::LessThanOrEq`

### `impl Std::U64 : Std::Mul`

### `impl Std::U64 : Std::Neg`

### `impl Std::U64 : Std::Rem`

### `impl Std::U64 : Std::Sub`

### `impl Std::U64 : Std::ToBytes`

### `impl Std::U64 : Std::ToString`

### `impl Std::U64 : Std::Zero`

### `impl Std::U8 : Std::Add`

### `impl Std::U8 : Std::Div`

### `impl Std::U8 : Std::Eq`

### `impl Std::U8 : Std::FromBytes`

### `impl Std::U8 : Std::FromString`

### `impl Std::U8 : Std::LessThan`

### `impl Std::U8 : Std::LessThanOrEq`

### `impl Std::U8 : Std::Mul`

### `impl Std::U8 : Std::Neg`

### `impl Std::U8 : Std::Rem`

### `impl Std::U8 : Std::Sub`

### `impl Std::U8 : Std::ToBytes`

### `impl Std::U8 : Std::ToString`

### `impl Std::U8 : Std::Zero`

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

### `loop_m : [m : Std::Monad] s -> (s -> m (Std::LoopResult s r)) -> m r`

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

### `undefined : () -> a`

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

## `namespace Std::Add`

### `add : [a : Std::Add] a -> a -> a`

Adds two values. An expression `x + y` is translated to `add(x, y)`.

## `namespace Std::Array`

### `@ : Std::I64 -> Std::Array a -> a`

Gets an element of an array at the specified index.

### `_get_ptr : Std::Array a -> Std::Ptr`

Get the pointer to the memory region where elements are stored.

This function is dangerous because if the array is not used after call of this function, the array will be deallocated soon and the returned pointer will be dangling.
Try using `borrow_ptr` instead.

### `_get_sub_size_asif : Std::I64 -> Std::I64 -> Std::I64 -> Std::I64 -> Std::Array a -> Std::Array a`

A function like `get_sub`, but behaves as if the size of the array is the specified value,

and has a parameter to specify additional capacity of the returned `Array`.

### `_sort_range_using_buffer : Std::Array a -> Std::I64 -> Std::I64 -> ((a, a) -> Std::Bool) -> Std::Array a -> (Std::Array a, Std::Array a)`

Sorts elements in a range of a vector by "less than" comparator.

This function receives a working buffer as the first argument to reduce memory allocation, and returns it as second element.

### `_unsafe_get : Std::I64 -> Std::Array a -> a`

Gets a value from an array, without bounds checking and retaining the returned value.

### `_unsafe_set : Std::I64 -> a -> Std::Array a -> Std::Array a`

Sets a value into an array, without uniqueness checking, bounds checking and releasing the old value.

### `_unsafe_set_size : Std::I64 -> Std::Array a -> Std::Array a`

Updates the length of an array, without uniqueness checking or validation of the given length value.

### `act : [f : Std::Functor] Std::I64 -> (a -> f a) -> Std::Array a -> f (Std::Array a)`

Modifies an array by a functorial action.

Semantically, `arr.act(idx, fun)` is equivalent to `fun(arr.@(idx)).map(|elm| arr.set(idx, elm))`.

This function can be defined for any functor `f` in general, but it is easier to understand the behavior when `f` is a monad:
the monadic action `act(idx, fun, arr)` first performs `fun(arr.@(idx))` to get a value `elm`, and returns a pure value `arr.set(idx, elm)`.

If you call `arr.act(idx, fun)` when both of `arr` and `arr.@(idx)` are unique, it is assured that `fun` receives the unique value.

If you call `act` on an array which is shared, this function clones the given array when inserting the result of your action into the array.
This means that you don't need to pay cloning cost when your action failed, as expected.

### `append : Std::Array a -> Std::Array a -> Std::Array a`

Appends an array to an array.

Note: Since `a1.append(a2)` puts `a2` after `a1`, `append(lhs, rhs)` puts `lhs` after `rhs`.

### `borrow_ptr : (Std::Ptr -> b) -> Std::Array a -> b`

Calls a function with a pointer to the memory region where elements are stored.

### `empty : Std::I64 -> Std::Array a`

Creates an empty array with specified capacity.

### `fill : Std::I64 -> a -> Std::Array a`

Creates an array of the specified length filled with the initial value.

The capacity is set to the same value as the length.

Example: `fill(n, x) == [x, x, x, ..., x]` (of length `n`).

### `find_by : (a -> Std::Bool) -> Std::Array a -> Std::Option Std::I64`

Finds the first index at which the element satisfies a condition.

### `force_unique : Std::Array a -> Std::Array a`

Force the uniqueness of an array.
If the given array is shared, this function returns the cloned array.

### `from_iter : Std::Iterator a -> Std::Array a`

Create an array from an iterator.

### `from_map : Std::I64 -> (Std::I64 -> a) -> Std::Array a`

Creates an array by a mapping function.

### `get_capacity : Std::Array a -> Std::I64`

Gets the capacity of an array.

### `get_first : Std::Array a -> Std::Option a`

Gets the first element of an array. Returns none if the array is empty.

### `get_last : Std::Array a -> Std::Option a`

Gets the last element of an array. Returns none if the array is empty.

### `get_size : Std::Array a -> Std::I64`

Gets the length of an array.

### `get_sub : Std::I64 -> Std::I64 -> Std::Array a -> Std::Array a`

`arr.get_sub(s, e)` returns an array `[ arr.@(i) | i ∈ [s, e) ]`,

More precisely, let `N` denote the the size of the `arr`.
Then `arr.get_sub(s, e)` returns `[ arr.@(s + i mod N) | i ∈ [0, n), n >= 0 is the minimum number such that s + n == e mod N ]`.

### `is_empty : Std::Array a -> Std::Bool`

Returns if the array is empty

### `mod : Std::I64 -> (a -> a) -> Std::Array a -> Std::Array a`

Updates an array by applying a function to the element at the specified index.

This function clones the given array if it is shared.

If you call `arr.mod(i, f)` when both of `arr` and `arr.@(i)` are unique, it is assured that `f` receives the element value which is unique.

### `pop_back : Std::Array a -> Std::Array a`

Pops an element at the back of an array.
If the array is empty, this function does nothing.

### `push_back : a -> Std::Array a -> Std::Array a`

Pushes an element to the back of an array.

### `reserve : Std::I64 -> Std::Array a -> Std::Array a`

Reserves the memory region for an array.

TODO: change to more optimized implementation.

### `set : Std::I64 -> a -> Std::Array a -> Std::Array a`

Updates an array by setting a value as the element at the specified index.

This function clones the given array if it is shared.

### `sort_by : ((a, a) -> Std::Bool) -> Std::Array a -> Std::Array a`

Sorts elements in a vector by "less than" comparator.

### `to_iter : Std::Array a -> Std::Iterator a`

Converts an array to an iterator.

### `truncate : Std::I64 -> Std::Array a -> Std::Array a`

Truncates an array, keeping the given number of first elements.

`truncante(len, arr)` does nothing if `len >= arr.get_size`.

## `namespace Std::Boxed`

### `@value : Std::Boxed a -> a`

Retrieves the field `value` from a value of `Boxed`.

### `act_value : [f : Std::Functor] (a -> f a) -> Std::Boxed a -> f (Std::Boxed a)`

Updates a value of `Boxed` by applying a functorial action to field `value`.

### `mod_value : (a -> a) -> Std::Boxed a -> Std::Boxed a`

Updates a value of `Boxed` by applying a function to field `value`.

### `set_value : a -> Std::Boxed a -> Std::Boxed a`

Updates a value of `Boxed` by setting field `value` to a specified one.

## `namespace Std::Debug`

### `_debug_print_to_stream : Std::IO::IOHandle -> Std::String -> ()`

Prints a string to the specified stream and flushes the stream.

### `assert : (() -> Std::String) -> Std::Bool -> ()`

Asserts that a condition (boolean value) is true. If the assertion failed, prints a message to the stderr and aborts the program.

### `assert_eq : [a : Std::Eq] (() -> Std::String) -> a -> a -> ()`

Asserts that two values are equal.
If the assertion failed, prints a message to the stderr and aborts the program.

### `assert_unique : (() -> Std::String) -> a -> a`

Asserts that the given value is unique, and returns the given value.
If the assertion failed, prints a message to the stderr and aborts the program.

The main use of this function is to check whether a boxed value given as an argument is unique.

### `consumed_time_while_io : Std::IO a -> Std::IO (a, Std::F64)`

Get clocks (cpu time) elapsed while executing an I/O action.

### `consumed_time_while_lazy : (() -> a) -> (a, Std::F64)`

Get clocks (cpu time) elapsed while evaluating a lazy value.

NOTE: This function is not pure and should only be used for temporary debugging purposes or for benchmarking.

### `debug_eprint : Std::String -> ()`

Prints a string to stderr and flushes.

NOTE: This function is not pure and should only be used for temporary debugging purposes or in test code.

### `debug_eprintln : Std::String -> ()`

Prints a string followed by a newline to stderr and flushes.

NOTE: This function is not pure and should only be used for temporary debugging purposes or in test code.

### `debug_print : Std::String -> ()`

Prints a string to stdout and flushes.

NOTE: This function is not pure and should only be used for temporary debugging purposes or in test code.

### `debug_println : Std::String -> ()`

Prints a string followed by a newline to stdout and flushes.

NOTE: This function is not pure and should only be used for temporary debugging purposes or in test code.

## `namespace Std::Div`

### `div : [a : Std::Div] a -> a -> a`

Divides a value by another value. An expression `x / y` is translated to `div(x, y)`.

## `namespace Std::Eq`

### `eq : [a : Std::Eq] a -> a -> Std::Bool`

Checks equality of two values. An expression `x == y` is translated to `eq(x, y)`.

## `namespace Std::F32`

### `abs : Std::F32 -> Std::F32`

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

### `to_string_exp : Std::F32 -> Std::String`

Converts a floating number to a string of exponential form.

### `to_string_exp_precision : Std::U8 -> Std::F32 -> Std::String`

Converts a floating number to a string of exponential form with specified precision (i.e., number of digits after the decimal point).

### `to_string_precision : Std::U8 -> Std::F32 -> Std::String`

Converts a floating number to a string with specified precision (i.e., number of digits after the decimal point).

## `namespace Std::F64`

### `abs : Std::F64 -> Std::F64`

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

### `to_string_exp : Std::F64 -> Std::String`

Converts a floating number to a string of exponential form.

### `to_string_exp_precision : Std::U8 -> Std::F64 -> Std::String`

Converts a floating number to a string of exponential form with specified precision (i.e., number of digits after the decimal point).

### `to_string_precision : Std::U8 -> Std::F64 -> Std::String`

Converts a floating number to a string with specified precision (i.e., number of digits after the decimal point).

## `namespace Std::FFI`

### `_unsafe_get_boxed_data_ptr : a -> Std::Ptr`

Returns a pointer to the data of a boxed value.

The difference from `unsafe_get_retained_ptr_of_boxed_value` is that this function returns a pointer to region where the payload of a boxed value is stored;
on the other hand, `unsafe_get_retained_ptr_of_boxed_value` returns a pointer to the boxed value itself (i.e., the control block of the value).

Note that if the call `v._unsafe_get_boxed_data_ptr` is the last usage of `v`, then this function deallocates `v` and returns a dangling pointer.
To avoid issues caused by this, use `unsafe_borrow_boxed_data_ptr` instead.

### `unsafe_borrow_boxed_data_ptr : (Std::Ptr -> b) -> a -> b`

Borrows a pointer to the data of a boxed value.

For more details, see the document of `_unsafe_get_boxed_data_ptr`.

### `unsafe_clear_errno : () -> ()`

Sets errno to zero.

### `unsafe_get_boxed_value_from_retained_ptr : Std::Ptr -> a`

Creates a boxed value from a retained pointer obtained by `unsafe_get_retained_ptr_of_boxed_value`.

### `unsafe_get_errno : () -> Std::I32`

Gets errno which is set by C functions.

### `unsafe_get_release_function_of_boxed_value : (() -> a) -> Std::Ptr`

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

### `unsafe_get_retain_function_of_boxed_value : (() -> a) -> Std::Ptr`

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

### `@_value : Std::FFI::Destructor a -> a`

Retrieves the field `_value` from a value of `Destructor`.

### `@dtor : Std::FFI::Destructor a -> a -> ()`

Retrieves the field `dtor` from a value of `Destructor`.

### `act__value : [f : Std::Functor] (a -> f a) -> Std::FFI::Destructor a -> f (Std::FFI::Destructor a)`

Updates a value of `Destructor` by applying a functorial action to field `_value`.

### `act_dtor : [f : Std::Functor] ((a -> ()) -> f (a -> ())) -> Std::FFI::Destructor a -> f (Std::FFI::Destructor a)`

Updates a value of `Destructor` by applying a functorial action to field `dtor`.

### `borrow : (a -> b) -> Std::FFI::Destructor a -> b`

Borrow the contained value.
`borrow(worker, dtor)` calls `worker` on the contained value captured by `dtor`, and returns the value returned by `worker`.
It is guaranteed that the `dtor` is alive during the call of `worker`.
In other words, the `worker` receives the contained value on which the destructor is not called yet.

### `make : a -> (a -> ()) -> Std::FFI::Destructor a`

Make a destructor value.

### `mod__value : (a -> a) -> Std::FFI::Destructor a -> Std::FFI::Destructor a`

Updates a value of `Destructor` by applying a function to field `_value`.

### `mod_dtor : ((a -> ()) -> a -> ()) -> Std::FFI::Destructor a -> Std::FFI::Destructor a`

Updates a value of `Destructor` by applying a function to field `dtor`.

### `set__value : a -> Std::FFI::Destructor a -> Std::FFI::Destructor a`

Updates a value of `Destructor` by setting field `_value` to a specified one.

### `set_dtor : (a -> ()) -> Std::FFI::Destructor a -> Std::FFI::Destructor a`

Updates a value of `Destructor` by setting field `dtor` to a specified one.

## `namespace Std::FromBytes`

### `from_bytes : [a : Std::FromBytes] Std::Array Std::U8 -> Std::Result Std::String a`

## `namespace Std::FromString`

### `from_string : [a : Std::FromString] Std::String -> Std::Result Std::String a`

## `namespace Std::Functor`

### `forget : [f : Std::Functor] f a -> f ()`

### `map : [f : Std::Functor] (a -> b) -> f a -> f b`

## `namespace Std::I16`

### `abs : Std::I16 -> Std::I16`

### `bit_and : Std::I16 -> Std::I16 -> Std::I16`

Calculates bitwise AND of two values.

### `bit_or : Std::I16 -> Std::I16 -> Std::I16`

Calculates bitwise OR of two values.

### `bit_xor : Std::I16 -> Std::I16 -> Std::I16`

Calculates bitwise XOR of two values.

### `maximum : Std::I16`

### `minimum : Std::I16`

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

### `abs : Std::I32 -> Std::I32`

### `bit_and : Std::I32 -> Std::I32 -> Std::I32`

Calculates bitwise AND of two values.

### `bit_or : Std::I32 -> Std::I32 -> Std::I32`

Calculates bitwise OR of two values.

### `bit_xor : Std::I32 -> Std::I32 -> Std::I32`

Calculates bitwise XOR of two values.

### `maximum : Std::I32`

### `minimum : Std::I32`

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

### `abs : Std::I64 -> Std::I64`

### `bit_and : Std::I64 -> Std::I64 -> Std::I64`

Calculates bitwise AND of two values.

### `bit_or : Std::I64 -> Std::I64 -> Std::I64`

Calculates bitwise OR of two values.

### `bit_xor : Std::I64 -> Std::I64 -> Std::I64`

Calculates bitwise XOR of two values.

### `maximum : Std::I64`

### `minimum : Std::I64`

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

### `abs : Std::I8 -> Std::I8`

### `bit_and : Std::I8 -> Std::I8 -> Std::I8`

Calculates bitwise AND of two values.

### `bit_or : Std::I8 -> Std::I8 -> Std::I8`

Calculates bitwise OR of two values.

### `bit_xor : Std::I8 -> Std::I8 -> Std::I8`

Calculates bitwise XOR of two values.

### `maximum : Std::I8`

### `minimum : Std::I8`

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

### `@_data : Std::IO a -> () -> a`

Retrieves the field `_data` from a value of `IO`.

### `_read_line_inner : Std::Bool -> Std::IO::IOHandle -> Std::IO::IOFail Std::String`

Reads characters from an IOHandle.

If the first argument `upto_newline` is true, this function reads a file upto newline or EOF.

### `_unsafe_perform : Std::IO a -> a`

Performs the I/O action. This may violate purity of Fix.

### `act__data : [f : Std::Functor] ((() -> a) -> f (() -> a)) -> Std::IO a -> f (Std::IO a)`

Updates a value of `IO` by applying a functorial action to field `_data`.

### `close_file : Std::IO::IOHandle -> Std::IO ()`

Closes a file.

Unlike C's `fclose`, closing an already closed `IOHandle` is safe and does nothing.

### `eprint : Std::String -> Std::IO ()`

Prints a string to stderr.

### `eprintln : Std::String -> Std::IO ()`

Prints a string followed by a newline to stderr.

### `exit : Std::I64 -> Std::IO a`

Exits the program with an error code.

### `exit_with_msg : Std::I64 -> Std::String -> Std::IO a`

Exits the program with an error message and an error code.

The error message is written to the standard error output.

### `from_func : (() -> a) -> Std::IO a`

Creates an IO action from a function.

### `get_arg : Std::I64 -> Std::IO (Std::Option Std::String)`

`get_arg(n)` returns the n-th (0-indexed) command line argument.
If n is greater than or equal to the number of command line arguments, this function returns none.

### `get_arg_count : Std::IO Std::I64`

Gets the number of command line arguments.

### `get_args : Std::IO (Std::Array Std::String)`

Gets command line arguments.

### `input_line : Std::IO Std::String`

Reads a line from stdin. If some error occurr, this function aborts the program.
If you want to handle errors, use `read_line(stdin)` instead.

### `is_eof : Std::IO::IOHandle -> Std::IO Std::Bool`

Checks if an `IOHandle` reached to the EOF.

### `loop_lines : Std::IO::IOHandle -> s -> (s -> Std::String -> Std::LoopResult s s) -> Std::IO::IOFail s`

Loop on lines read from an `IOHandle`.

`loop_lines(handle, initial_state, worker)` calls `worker` on the pair of current state and a line string read from `handle`.
The function `worker` should return an updated state as `LoopResult` value, i.e., a value created by `continue` or `break`.
When the `handle` reaches to the EOF or `worker` returns a `break` value, `loop_lines` returns the last state value.

Note that the line string passed to `worker` may contain a newline code at the end. To remove it, use `String::strip_last_spaces`.

### `loop_lines_io : Std::IO::IOHandle -> s -> (s -> Std::String -> Std::IO::IOFail (Std::LoopResult s s)) -> Std::IO::IOFail s`

Loop on lines read from an `IOHandle`.

Similar to `loop_lines`, but the worker function can perform an IO action.

### `mod__data : ((() -> a) -> () -> a) -> Std::IO a -> Std::IO a`

Updates a value of `IO` by applying a function to field `_data`.

### `open_file : Std::Path -> Std::String -> Std::IO::IOFail Std::IO::IOHandle`

Openes a file. The second argument is a mode string for `fopen` C function.

### `print : Std::String -> Std::IO ()`

Prints a string to stdout.

### `println : Std::String -> Std::IO ()`

Prints a string followed by a newline to stdout.

### `read_bytes : Std::IO::IOHandle -> Std::IO::IOFail (Std::Array Std::U8)`

Reads all bytes from an IOHandle.

### `read_file_bytes : Std::Path -> Std::IO::IOFail (Std::Array Std::U8)`

Reads all bytes from a file.

### `read_file_string : Std::Path -> Std::IO::IOFail Std::String`

Raads all characters from a file.

### `read_line : Std::IO::IOHandle -> Std::IO::IOFail Std::String`

Reads characters from a IOHandle upto newline or EOF.
The returned string may include newline at its end.

### `read_n_bytes : Std::IO::IOHandle -> Std::I64 -> Std::IO::IOFail (Std::Array Std::U8)`

Reads at most n bytes from an IOHandle.

### `read_string : Std::IO::IOHandle -> Std::IO::IOFail Std::String`

Reads all characters from an IOHandle.

### `set__data : (() -> a) -> Std::IO a -> Std::IO a`

Updates a value of `IO` by setting field `_data` to a specified one.

### `stderr : Std::IO::IOHandle`

The handle for standard error.

### `stdin : Std::IO::IOHandle`

The handle for standard input.

### `stdout : Std::IO::IOHandle`

The handle for standard output.

### `with_file : Std::Path -> Std::String -> (Std::IO::IOHandle -> Std::IO::IOFail a) -> Std::IO::IOFail a`

Performs a function with a file handle. The second argument is a mode string for `fopen` C function.

The file handle will be closed automatically.

### `write_bytes : Std::IO::IOHandle -> Std::Array Std::U8 -> Std::IO::IOFail ()`

Writes a byte array into an IOHandle.

### `write_file_bytes : Std::Path -> Std::Array Std::U8 -> Std::IO::IOFail ()`

Writes a byte array into a file.

### `write_file_string : Std::Path -> Std::String -> Std::IO::IOFail ()`

Writes a string into a file.

### `write_string : Std::IO::IOHandle -> Std::String -> Std::IO::IOFail ()`

Writes a string into an IOHandle.

## `namespace Std::IO::IOFail`

### `@_data : Std::IO::IOFail a -> Std::IO (Std::Result Std::String a)`

Retrieves the field `_data` from a value of `IOFail`.

### `act__data : [f : Std::Functor] (Std::IO (Std::Result Std::String a) -> f (Std::IO (Std::Result Std::String a))) -> Std::IO::IOFail a -> f (Std::IO::IOFail a)`

Updates a value of `IOFail` by applying a functorial action to field `_data`.

### `from_result : Std::Result Std::String a -> Std::IO::IOFail a`

Creates an pure `IOFail` value from a `Result` value.

### `lift : Std::IO a -> Std::IO::IOFail a`

Lifts an `IO` action to a successful `IOFail` action.

### `mod__data : (Std::IO (Std::Result Std::String a) -> Std::IO (Std::Result Std::String a)) -> Std::IO::IOFail a -> Std::IO::IOFail a`

Updates a value of `IOFail` by applying a function to field `_data`.

### `set__data : Std::IO (Std::Result Std::String a) -> Std::IO::IOFail a -> Std::IO::IOFail a`

Updates a value of `IOFail` by setting field `_data` to a specified one.

### `throw : Std::String -> Std::IO::IOFail a`

Creates an error `IOFail` action.

### `to_result : Std::IO::IOFail a -> Std::IO (Std::Result Std::String a)`

Converts an `IOFail` to an `Result` value (wrapped by `IO`).

### `try : (Std::String -> Std::IO a) -> Std::IO::IOFail a -> Std::IO a`

Converts an `IOFail` value to an `IO` value by an error handler (i.e., a `catch`) function.

## `namespace Std::IO::IOHandle`

### `@_data : Std::IO::IOHandle -> Std::FFI::Destructor Std::Ptr`

Retrieves the field `_data` from a value of `IOHandle`.

### `_file_ptr : Std::IO::IOHandle -> Std::Ptr`

Gets pointer to C's `FILE` value from an `IOHandle`.

DO NOT call `fclose` on the pointer returned by this function.
To close an `IOHandle`, use `IO::close_file`.

### `_unsafe_close : Std::IO::IOHandle -> ()`

Closes an `IOHandle`.

This is an I/O action not wrapped by `IO`; use `IO::close_file` in the usual case.

### `act__data : [f : Std::Functor] (Std::FFI::Destructor Std::Ptr -> f (Std::FFI::Destructor Std::Ptr)) -> Std::IO::IOHandle -> f Std::IO::IOHandle`

Updates a value of `IOHandle` by applying a functorial action to field `_data`.

### `from_file_ptr : Std::Ptr -> Std::IO::IOHandle`

Creates an `IOHandle` from a file pointer (i.e., pointer to C's `FILE`).

Creating two `IOHandle`s from a single file pointer is forbidden.

### `mod__data : (Std::FFI::Destructor Std::Ptr -> Std::FFI::Destructor Std::Ptr) -> Std::IO::IOHandle -> Std::IO::IOHandle`

Updates a value of `IOHandle` by applying a function to field `_data`.

### `set__data : Std::FFI::Destructor Std::Ptr -> Std::IO::IOHandle -> Std::IO::IOHandle`

Updates a value of `IOHandle` by setting field `_data` to a specified one.

## `namespace Std::Iterator`

### `@next : Std::Iterator a -> () -> Std::Option (a, Std::Iterator a)`

Retrieves the field `next` from a value of `Iterator`.

### `_flatten : Std::Iterator (Std::Iterator a) -> Std::Iterator a`

Flatten an iterator of iterators.

You should use `Monad::flatten` instead of this function.
This function is used in the implementation of `Monad::bind` for `Iterator`.

### `_flatten_sub : Std::Iterator a -> Std::Iterator (Std::Iterator a) -> Std::Iterator a`

### `act_next : [f : Std::Functor] ((() -> Std::Option (a, Std::Iterator a)) -> f (() -> Std::Option (a, Std::Iterator a))) -> Std::Iterator a -> f (Std::Iterator a)`

Updates a value of `Iterator` by applying a functorial action to field `next`.

### `advance : Std::Iterator a -> Std::Option (a, Std::Iterator a)`

Gets next value and next iterator.

### `append : Std::Iterator a -> Std::Iterator a -> Std::Iterator a`

Appends an iterator to a iterator.
Note: Since `iter1.append(iter2)` puts `iter2` after `iter1`, `append(lhs, rhs)` puts `lhs` after `rhs`.

### `bang : Std::Iterator a -> Std::Iterator a`

Evaluates all elements of iterator.
TODO: add test

### `count_up : Std::I64 -> Std::Iterator Std::I64`

Creates an iterator that counts up from a number.
count_up(n) = [n, n+1, n+2, ...]

### `empty : Std::Iterator a`

Creates an empty iterator.

### `filter : (a -> Std::Bool) -> Std::Iterator a -> Std::Iterator a`

Filters elements by a condition function

### `find_last : Std::Iterator a -> Std::Option a`

Finds the last element of an iterator.

### `fold : b -> (b -> a -> b) -> Std::Iterator a -> b`

Folds iterator from left to right.
Example: `fold(init, op, [a0, a1, a2, ...]) = ...op(op(op(init, a0), a1), a2)...`

### `fold_m : [m : Std::Monad] b -> (b -> a -> m b) -> Std::Iterator a -> m b`

Folds iterator from left to right by monadic action.

### `from_array : Std::Array a -> Std::Iterator a`

Creates iterator from an array.

### `from_map : (Std::I64 -> a) -> Std::Iterator a`

Creates iterator from mapping function.
from_map(f) = [f(0), f(1), f(2), ...]

### `generate : s -> (s -> Std::Option (a, s)) -> Std::Iterator a`

Generates an iterator from a state transition function.
- if `f(s)` is none, `generate(s, f)` is empty.
- if `f(s)` is some value `(e, s1)`, then `generate(s, f)` starts by `e` followed by `generate(s2, f)`.

### `get_first : Std::Iterator a -> Std::Option a`

Gets the first element of an iterator. If the iterator is empty, this function returns `none`.
TODO: add test

### `get_size : Std::Iterator a -> Std::I64`

Counts the number of elements of an iterator.

### `get_tail : Std::Iterator a -> Std::Option (Std::Iterator a)`

Removes the first element from an iterator. If the iterator is empty, this function returns `none`.
TODO: add test

### `intersperse : a -> Std::Iterator a -> Std::Iterator a`

Intersperse an elemnt between elements of an iterator.

Example:
```
Iterator::from_array([1,2,3]).intersperse(0) == Iterator::from_array([1,0,2,0,3])
```

### `is_empty : Std::Iterator a -> Std::Bool`

Check if the iterator is empty.

### `loop_iter : b -> (b -> a -> Std::LoopResult b b) -> Std::Iterator a -> b`

Loop along an iterator. At each iteration step, you can choose to continue or to break.

### `loop_iter_m : [m : Std::Monad] b -> (b -> a -> m (Std::LoopResult b b)) -> Std::Iterator a -> m b`

Loop by monadic action along an iterator. At each iteration step, you can choose to continue or to break.

### `mod_next : ((() -> Std::Option (a, Std::Iterator a)) -> () -> Std::Option (a, Std::Iterator a)) -> Std::Iterator a -> Std::Iterator a`

Updates a value of `Iterator` by applying a function to field `next`.

### `product : Std::Iterator a -> Std::Iterator b -> Std::Iterator (b, a)`

Generates the cartesian product of two iterators.

Example: `[1, 2, 3].to_iter.product(['a', 'b'].to_iter).to_array == [(1, 'a'), (2, 'a'), (3, 'a'), (1, 'b'), (2, 'b'), (3, 'b')]`

### `push_front : a -> Std::Iterator a -> Std::Iterator a`

Pushes an element to an iterator.

### `range : Std::I64 -> Std::I64 -> Std::Iterator Std::I64`

Creates a range iterator, i.e. an iterator of the form `[a, a+1, a+2, ..., b-1]`.

### `reverse : Std::Iterator a -> Std::Iterator a`

Reverses an iterator.

### `set_next : (() -> Std::Option (a, Std::Iterator a)) -> Std::Iterator a -> Std::Iterator a`

Updates a value of `Iterator` by setting field `next` to a specified one.

### `subsequences : Std::Iterator a -> Std::Iterator (Std::Iterator a)`

Generates all subsequences of an iterator.

`[1,2,3].to_iter.subsequences` is `[[], [3], [2], [2, 3], [1], [1, 3], [1, 2], [1, 2, 3]].to_iter.map(to_iter)`.

### `sum : [a : Std::Additive] Std::Iterator a -> a`

Calculates the sum of elements of an iterator.

### `take : Std::I64 -> Std::Iterator a -> Std::Iterator a`

Takes at most n elements from an iterator.

### `take_while : (a -> Std::Bool) -> Std::Iterator a -> Std::Iterator a`

Take elements of an iterator while a condition is satisfied.
TODO: add test

### `to_array : Std::Iterator a -> Std::Array a`

Convert an iterator to an array.

### `zip : Std::Iterator b -> Std::Iterator a -> Std::Iterator (a, b)`

Zip two iterators.

## `namespace Std::LessThan`

### `less_than : [a : Std::LessThan] a -> a -> Std::Bool`

Compares two values. An expression `x < y` is translated to `less_than(x, y)`.

### `max : [a : Std::LessThan] a -> a -> a`

### `min : [a : Std::LessThan] a -> a -> a`

## `namespace Std::LessThanOrEq`

### `less_than_or_eq : [a : Std::LessThanOrEq] a -> a -> Std::Bool`

Compares two values. An expression `x <= y` is translated to `less_than_or_eq(x, y)`.

## `namespace Std::LoopResult`

### `as_break : Std::LoopResult s b -> b`

Unwraps a union value of `LoopResult` as the variant `break`.
If the value is not the variant `break`, this function aborts the program.

### `as_continue : Std::LoopResult s b -> s`

Unwraps a union value of `LoopResult` as the variant `continue`.
If the value is not the variant `continue`, this function aborts the program.

### `break : b -> Std::LoopResult s b`

Constructs a value of union `LoopResult` taking the variant `break`.

### `break_m : [m : Std::Monad] r -> m (Std::LoopResult s r)`

Make a break value wrapped in a monad.

This is used with `loop_m` function.

### `continue : s -> Std::LoopResult s b`

Constructs a value of union `LoopResult` taking the variant `continue`.

### `continue_m : [m : Std::Monad] s -> m (Std::LoopResult s r)`

Make a continue value wrapped in a monad.

This is used with `loop_m` function.

### `is_break : Std::LoopResult s b -> Std::Bool`

Checks if a union value of `LoopResult` is the variant `break`.

### `is_continue : Std::LoopResult s b -> Std::Bool`

Checks if a union value of `LoopResult` is the variant `continue`.

### `mod_break : (b -> b) -> Std::LoopResult s b -> Std::LoopResult s b`

Updates a value of union `LoopResult` by applying a function if it is the variant `break`, or doing nothing otherwise.

### `mod_continue : (s -> s) -> Std::LoopResult s b -> Std::LoopResult s b`

Updates a value of union `LoopResult` by applying a function if it is the variant `continue`, or doing nothing otherwise.

## `namespace Std::Monad`

### `bind : [m : Std::Monad] (a -> m b) -> m a -> m b`

### `flatten : [m : Std::Monad] m (m a) -> m a`

Flattens a nested monadic action.

### `pure : [m : Std::Monad] a -> m a`

### `unless : [m : Std::Monad] Std::Bool -> m () -> m ()`

`unless(cond, act)` where `act` is a monadic value which returns `()` perfoms `act` only when `cond` is false.

### `when : [m : Std::Monad] Std::Bool -> m () -> m ()`

`when(cond, act)` where `act` is a monadic value which returns `()` perfoms `act` only when `cond` is true.

## `namespace Std::Mul`

### `mul : [a : Std::Mul] a -> a -> a`

Multiplies a value by another value. An expression `x * y` is translated to `mul(x, y)`.

## `namespace Std::Neg`

### `neg : [a : Std::Neg] a -> a`

Negates a value. An expression `-x` is translated to `neg(x)`.

## `namespace Std::Not`

### `not : [a : Std::Not] a -> a`

Logical NOT of a value. An expression `!x` is translated to `not(x)`.

## `namespace Std::Option`

### `as_none : Std::Option a -> ()`

Unwraps a union value of `Option` as the variant `none`.
If the value is not the variant `none`, this function aborts the program.

### `as_some : Std::Option a -> a`

Unwraps a union value of `Option` as the variant `some`.
If the value is not the variant `some`, this function aborts the program.

### `as_some_or : a -> Std::Option a -> a`

Unwrap an option value if it is `some`, or returns given default value if it is `none`.

### `is_none : Std::Option a -> Std::Bool`

Checks if a union value of `Option` is the variant `none`.

### `is_some : Std::Option a -> Std::Bool`

Checks if a union value of `Option` is the variant `some`.

### `map_or : b -> (a -> b) -> Std::Option a -> b`

Returns the provided default value if the option is none, or applies a function to the contained value if the option is some.

### `mod_none : (() -> ()) -> Std::Option a -> Std::Option a`

Updates a value of union `Option` by applying a function if it is the variant `none`, or doing nothing otherwise.

### `mod_some : (a -> a) -> Std::Option a -> Std::Option a`

Updates a value of union `Option` by applying a function if it is the variant `some`, or doing nothing otherwise.

### `none : () -> Std::Option a`

Constructs a value of union `Option` taking the variant `none`.

### `some : a -> Std::Option a`

Constructs a value of union `Option` taking the variant `some`.

## `namespace Std::Path`

### `@_data : Std::Path -> Std::String`

Retrieves the field `_data` from a value of `Path`.

### `act__data : [f : Std::Functor] (Std::String -> f Std::String) -> Std::Path -> f Std::Path`

Updates a value of `Path` by applying a functorial action to field `_data`.

### `mod__data : (Std::String -> Std::String) -> Std::Path -> Std::Path`

Updates a value of `Path` by applying a function to field `_data`.

### `parse : Std::String -> Std::Option Std::Path`

Parse a string.

### `set__data : Std::String -> Std::Path -> Std::Path`

Updates a value of `Path` by setting field `_data` to a specified one.

## `namespace Std::Ptr`

### `add_offset : Std::I64 -> Std::Ptr -> Std::Ptr`

Adds an offset to a pointer.

### `subtract_ptr : Std::Ptr -> Std::Ptr -> Std::I64`

Subtracts two pointers.

Note that `x.subtract_ptr(y)` calculates `x - y`, so `subtract_ptr(x, y)` calculates `y - x`.

## `namespace Std::PunchedArray`

### `@_data : Std::PunchedArray a -> Std::FFI::Destructor (Std::Array a)`

Retrieves the field `_data` from a value of `PunchedArray`.

### `@idx : Std::PunchedArray a -> Std::I64`

Retrieves the field `idx` from a value of `PunchedArray`.

### `act__data : [f : Std::Functor] (Std::FFI::Destructor (Std::Array a) -> f (Std::FFI::Destructor (Std::Array a))) -> Std::PunchedArray a -> f (Std::PunchedArray a)`

Updates a value of `PunchedArray` by applying a functorial action to field `_data`.

### `act_idx : [f : Std::Functor] (Std::I64 -> f Std::I64) -> Std::PunchedArray a -> f (Std::PunchedArray a)`

Updates a value of `PunchedArray` by applying a functorial action to field `idx`.

### `mod__data : (Std::FFI::Destructor (Std::Array a) -> Std::FFI::Destructor (Std::Array a)) -> Std::PunchedArray a -> Std::PunchedArray a`

Updates a value of `PunchedArray` by applying a function to field `_data`.

### `mod_idx : (Std::I64 -> Std::I64) -> Std::PunchedArray a -> Std::PunchedArray a`

Updates a value of `PunchedArray` by applying a function to field `idx`.

### `plug_in : a -> Std::PunchedArray a -> Std::Array a`

Plug in an element to a punched array to get back an array.

### `set__data : Std::FFI::Destructor (Std::Array a) -> Std::PunchedArray a -> Std::PunchedArray a`

Updates a value of `PunchedArray` by setting field `_data` to a specified one.

### `set_idx : Std::I64 -> Std::PunchedArray a -> Std::PunchedArray a`

Updates a value of `PunchedArray` by setting field `idx` to a specified one.

### `unsafe_punch : Std::I64 -> Std::Array a -> (Std::PunchedArray a, a)`

Creates a punched array by moving out the element at the specified index.

NOTE: this function assumes that the given array is unique WITHOUT CHECKING.
The uniqueness of the array is ensured in the `Array::act` function.

## `namespace Std::Rem`

### `rem : [a : Std::Rem] a -> a -> a`

Calculate remainder of a value dividing another value. An expression `x % y` is translated to `rem(x, y)`.

## `namespace Std::Result`

### `as_err : Std::Result e o -> e`

Unwraps a union value of `Result` as the variant `err`.
If the value is not the variant `err`, this function aborts the program.

### `as_ok : Std::Result e o -> o`

Unwraps a union value of `Result` as the variant `ok`.
If the value is not the variant `ok`, this function aborts the program.

### `err : e -> Std::Result e o`

Constructs a value of union `Result` taking the variant `err`.

### `is_err : Std::Result e o -> Std::Bool`

Checks if a union value of `Result` is the variant `err`.

### `is_ok : Std::Result e o -> Std::Bool`

Checks if a union value of `Result` is the variant `ok`.

### `mod_err : (e -> e) -> Std::Result e o -> Std::Result e o`

Updates a value of union `Result` by applying a function if it is the variant `err`, or doing nothing otherwise.

### `mod_ok : (o -> o) -> Std::Result e o -> Std::Result e o`

Updates a value of union `Result` by applying a function if it is the variant `ok`, or doing nothing otherwise.

### `ok : o -> Std::Result e o`

Constructs a value of union `Result` taking the variant `ok`.

### `unwrap : Std::Result e o -> o`

Returns the containing value if the value is ok, or otherwise aborts the program.

## `namespace Std::String`

### `@_data : Std::String -> Std::Array Std::U8`

Retrieves the field `_data` from a value of `String`.

### `_get_c_str : Std::String -> Std::Ptr`

Get the null-terminated C string.

Note that in case the string is not used after call of this function, the returned pointer will be already released.

### `_unsafe_from_c_str : Std::Array Std::U8 -> Std::String`

Create a string from C string (i.e., null-terminated byte array).

If the byte array doesn't include `\0`, this function causes undefined behavior.

### `_unsafe_from_c_str_ptr : Std::Ptr -> Std::String`

Create a `String` from a pointer to null-terminated C string.

If `ptr` is not pointing to a valid null-terminated C string, this function cause undefined behavior.

### `act__data : [f : Std::Functor] (Std::Array Std::U8 -> f (Std::Array Std::U8)) -> Std::String -> f Std::String`

Updates a value of `String` by applying a functorial action to field `_data`.

### `borrow_c_str : (Std::Ptr -> a) -> Std::String -> a`

Call a function with a null-terminated C string.

### `concat : Std::String -> Std::String -> Std::String`

Concatenate two strings.

Note: Since `s1.concat(s2)` puts `s2` after `s1`, `concat(lhs, rhs)` puts `lhs` after `rhs`.

### `concat_iter : Std::Iterator Std::String -> Std::String`

Concatenate an iterator of strings.

### `empty : Std::I64 -> Std::String`

Create an empty string, which is reserved for a length.

### `find : Std::String -> Std::I64 -> Std::String -> Std::Option Std::I64`

`str.find(token, start_idx)` finds the index where `token` firstly appears in `str`, starting from `start_idx`.

Note that this function basically returns a number less than or equal to `start_idx`, but there is an exception:
`str.find("", start_idx)` with `start_idx >= str.get_size` returns `str.get_size`, not `start_idx`.

### `get_bytes : Std::String -> Std::Array Std::U8`

Gets the byte array of a string, containing null-terminator.

### `get_first_byte : Std::String -> Std::Option Std::U8`

Gets the first byte of a string. Returns none if the string is empty.

### `get_last_byte : Std::String -> Std::Option Std::U8`

Gets the last byte of a string. Returns none if the string is empty.

### `get_size : Std::String -> Std::I64`

Gets the length of a string.

### `get_sub : Std::I64 -> Std::I64 -> Std::String -> Std::String`

`String` version of `Array::get_sub`.

### `is_empty : Std::String -> Std::Bool`

Returns if the string is empty or not.

### `join : Std::String -> Std::Iterator Std::String -> Std::String`

Joins strings by a separator.

### `mod__data : (Std::Array Std::U8 -> Std::Array Std::U8) -> Std::String -> Std::String`

Updates a value of `String` by applying a function to field `_data`.

### `pop_back_byte : Std::String -> Std::String`

Removes the last byte.
If the string is empty, this function does nothing.

### `set__data : Std::Array Std::U8 -> Std::String -> Std::String`

Updates a value of `String` by setting field `_data` to a specified one.

### `split : Std::String -> Std::String -> Std::Iterator Std::String`

`str.split(sep)` splits `str` by `sep` into an iterator.
- If `sep` is empty, this function returns an infinite sequence of ""s.
- If `sep` is non-empty and `str` is empty, this function returns an iterator with a single element "".

### `strip_first_bytes : (Std::U8 -> Std::Bool) -> Std::String -> Std::String`

Removes the first byte of a string while it satisifies the specified condition.

### `strip_first_spaces : Std::String -> Std::String`

Removes leading whitespace characters.

### `strip_last_bytes : (Std::U8 -> Std::Bool) -> Std::String -> Std::String`

Removes the last byte of a string while it satisifies the specified condition.

### `strip_last_newlines : Std::String -> Std::String`

Removes newlines and carriage returns at the end of the string.

### `strip_last_spaces : Std::String -> Std::String`

Removes trailing whitespace characters.

### `strip_spaces : Std::String -> Std::String`

Strips leading and trailing whitespace characters.

## `namespace Std::Sub`

### `sub : [a : Std::Sub] a -> a -> a`

Subtracts a value from another value. An expression `x - y` is translated to `sub(x, y)`.

## `namespace Std::ToBytes`

### `to_bytes : [a : Std::ToBytes] a -> Std::Array Std::U8`

## `namespace Std::ToString`

### `to_string : [a : Std::ToString] a -> Std::String`

## `namespace Std::Tuple2`

### `@0 : (t0, t1) -> t0`

Retrieves the field `0` from a value of `Tuple2`.

### `@1 : (t0, t1) -> t1`

Retrieves the field `1` from a value of `Tuple2`.

### `act_0 : [f : Std::Functor] (t0 -> f t0) -> (t0, t1) -> f (t0, t1)`

Updates a value of `Tuple2` by applying a functorial action to field `0`.

### `act_1 : [f : Std::Functor] (t1 -> f t1) -> (t0, t1) -> f (t0, t1)`

Updates a value of `Tuple2` by applying a functorial action to field `1`.

### `mod_0 : (t0 -> t0) -> (t0, t1) -> (t0, t1)`

Updates a value of `Tuple2` by applying a function to field `0`.

### `mod_1 : (t1 -> t1) -> (t0, t1) -> (t0, t1)`

Updates a value of `Tuple2` by applying a function to field `1`.

### `set_0 : t0 -> (t0, t1) -> (t0, t1)`

Updates a value of `Tuple2` by setting field `0` to a specified one.

### `set_1 : t1 -> (t0, t1) -> (t0, t1)`

Updates a value of `Tuple2` by setting field `1` to a specified one.

## `namespace Std::Tuple3`

### `@0 : (t0, t1, t2) -> t0`

Retrieves the field `0` from a value of `Tuple3`.

### `@1 : (t0, t1, t2) -> t1`

Retrieves the field `1` from a value of `Tuple3`.

### `@2 : (t0, t1, t2) -> t2`

Retrieves the field `2` from a value of `Tuple3`.

### `act_0 : [f : Std::Functor] (t0 -> f t0) -> (t0, t1, t2) -> f (t0, t1, t2)`

Updates a value of `Tuple3` by applying a functorial action to field `0`.

### `act_1 : [f : Std::Functor] (t1 -> f t1) -> (t0, t1, t2) -> f (t0, t1, t2)`

Updates a value of `Tuple3` by applying a functorial action to field `1`.

### `act_2 : [f : Std::Functor] (t2 -> f t2) -> (t0, t1, t2) -> f (t0, t1, t2)`

Updates a value of `Tuple3` by applying a functorial action to field `2`.

### `mod_0 : (t0 -> t0) -> (t0, t1, t2) -> (t0, t1, t2)`

Updates a value of `Tuple3` by applying a function to field `0`.

### `mod_1 : (t1 -> t1) -> (t0, t1, t2) -> (t0, t1, t2)`

Updates a value of `Tuple3` by applying a function to field `1`.

### `mod_2 : (t2 -> t2) -> (t0, t1, t2) -> (t0, t1, t2)`

Updates a value of `Tuple3` by applying a function to field `2`.

### `set_0 : t0 -> (t0, t1, t2) -> (t0, t1, t2)`

Updates a value of `Tuple3` by setting field `0` to a specified one.

### `set_1 : t1 -> (t0, t1, t2) -> (t0, t1, t2)`

Updates a value of `Tuple3` by setting field `1` to a specified one.

### `set_2 : t2 -> (t0, t1, t2) -> (t0, t1, t2)`

Updates a value of `Tuple3` by setting field `2` to a specified one.

## `namespace Std::U16`

### `bit_and : Std::U16 -> Std::U16 -> Std::U16`

Calculates bitwise AND of two values.

### `bit_or : Std::U16 -> Std::U16 -> Std::U16`

Calculates bitwise OR of two values.

### `bit_xor : Std::U16 -> Std::U16 -> Std::U16`

Calculates bitwise XOR of two values.

### `maximum : Std::U16`

### `minimum : Std::U16`

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

### `maximum : Std::U32`

### `minimum : Std::U32`

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

### `maximum : Std::U64`

### `minimum : Std::U64`

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

### `maximum : Std::U8`

### `minimum : Std::U8`

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

## `namespace Std::Zero`

### `zero : [a : Std::Zero] a`