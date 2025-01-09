# `module Std`

Module `Std` provides basic types, traits and values.

This module is special in the sense that:
- It is always imported implicitly. If you don't want to import some or all of entities in this module, you should write `import Std {...entities...}` explicitly.
- It contains built-in types or values which are defined or implemented directly by Fix compiler, not by Fix source code.

NOTE on tuples:
The tuple types `Std::TupleN` are defined on demand, i.e., if the user uses N-tuple in the source code,
the compiler generates definition `TupleN` and related functions / trait implementations.
The document for `Std` module describes about them up to N=3, but you can use larger tuples in the same way.

# Types and aliases

## `namespace Std`

### `type Array a = box { built-in }`

The type of variable length arrays. This is a boxed type.

### `type Arrow a b = unbox { built-in }`

`Arrow a b` represents the type of a function that takes a value of type `a` and returns a value of type `b`. Usually written as `a -> b`.

### `type Bool = unbox { built-in }`

The type of boolean values.

### `type Box a = box struct { ...fields... }`

Boxed wrapper for a type.

#### field `value : a`

### `type ErrMsg = Std::String`

A type (alias) for error message.

### `type F32 = unbox { built-in }`

The type of 32-bit floating point values.

### `type F64 = unbox { built-in }`

The type of 64-bit floating point values.

### `type I16 = unbox { built-in }`

The type of 16-bit signed integers.

### `type I32 = unbox { built-in }`

The type of 32-bit signed integers.

### `type I64 = unbox { built-in }`

The type of 64-bit signed integers.

### `type I8 = unbox { built-in }`

The type of 8-bit signed integers.

### `type IO a = unbox struct { ...fields... }`

`IO a` is a type representing I/O actions which return values of type `a`.

#### field `runner : Std::IO::IOState -> (Std::IO::IOState, a)`

### `type Iterator a = unbox struct { ...fields... }`

The type of iterators (a.k.a lazy list).

#### field `next : () -> Std::Option (a, Std::Iterator a)`

### `type Lazy = () -> a`

The type of lazily generated values.

You can create a lazy value by `|_| (...an expression to generate the value...)`,
and you can evaluate a lazy value `v` by `v()`.

### `type LoopState s r = unbox union { ...variants... }`

A union type with variants `continue` and `break`.

This type is used to represent the result of a loop body function passed to `Std::loop` or other similar functions.

#### variant `continue : s`

#### variant `break : r`

### `type Option a = unbox union { ...variants... }`

#### variant `none : ()`

#### variant `some : a`

### `type Path = unbox struct { ...fields... }`

The type for file path.

TODO: give better implementation.

#### field `_data : Std::String`

### `type Ptr = unbox { built-in }`

The type of pointers.

### `type PunchedArray a = unbox struct { ...fields... }`

The type of punched arrays.

A punched array is an array from which a certain element has been removed.
This is used in the implementation of `Array::act`.

#### field `_arr : Std::Array a`

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

### `type U16 = unbox { built-in }`

The type of 16-bit unsigned integers.

### `type U32 = unbox { built-in }`

The type of 32-bit unsigned integers.

### `type U64 = unbox { built-in }`

The type of 64-bit unsigned integers.

### `type U8 = unbox { built-in }`

The type of 8-bit unsinged integers.

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

`Destructor a` is a wrapper type for `a`, which can have a destructor function `a -> IO a`.
Just before a value of type `Destructor a` is dropped, the destructor function is called on the contained value, and the value can be modified by the `IO` action.

This type is used to create a Fix's type that wraps a resource allocated by FFI. In such cases, the destructor release the resource by FFI.

NOTE: In the destructor, only IO actions for finalizing the passed value are allowed, and you should not perform other IO actions such as writing standard output.

NOTE: Of course, if the value stored in `Destructor` also exists outside of `Destructor`, the value still exists in the Fix program even after the destructor function is called,
and there is a possibility that the value is used after the destructor function is called.

#### field `_value : a`

#### field `dtor : a -> Std::IO a`

## `namespace Std::IO`

### `type IOFail a = unbox struct { ...fields... }`

The type for I/O actions which may fail.

#### field `_data : Std::IO (Std::Result Std::String a)`

### `type IOHandle = unbox struct { ...fields... }`

A handle type for read / write operations on files, stdin, stdout, stderr.

You can create `IOHandle` value by `IO::open_file`, and close it by `IO::close_file`.
There are also global `IO::IOHandle::stdin`, `IO::IOHandle::stdout`, `IO::IOHandle::stderr`.

`IOHandle` is different from C's `FILE` structure in that it is safe to close it twice.
If you try to get a file pointer by `file_ptr` from a closed `IOHandle`, you will get `nullptr`.

NOTE:
`IOHandle` is implemented by `Destructor`, but the destructor function does not close the file pointer.
(The destructor function only frees the management memory area.)
You should explicitly close the file pointer by `IO::close_file`.

#### field `_data : Std::FFI::Destructor Std::Ptr`

### `type IOState = unbox { built-in }`

The type of the "state"s modified by I/O operations. 

The type `IO a` is isomorphic to `IOState -> (IOState, a)`.

Values of type `IOState` must be used linearly, i.e., each value must be used exactly once and must not be duplicated or discarded.

Values of type `IOState` are generated by the runtime when executing `IO` actions like `main` and passed linearly to various places in the program. At some places, `IOState` values are consumed by `FFI_CALL_IOS` expressions and new `IOState` values are generated. When `IO` actions like `main` finish, they are consumed by the runtime and disappear.

Technically, `IOState` exists to specify the execution of I/O operations to the optimizer in the compiler.

# Traits and aliases

## `namespace Std`

### `trait a : Add`

Trait for infix operator `+`.

#### method `add : a -> a -> a`

Adds two values. An expression `x + y` is translated to `add(x, y)`.

### `trait a : Boxed`

Marker trait for boxed types.

This trait is automatically implemented for all boxed types.
Implementing this trait manually is not allowed.

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

### `impl [t0 : Std::Eq, t1 : Std::Eq] (t0, t1) : Std::Eq`

### `impl [t0 : Std::Eq, t0 : Std::LessThan, t1 : Std::Eq, t1 : Std::LessThan] (t0, t1) : Std::LessThan`

### `impl [t0 : Std::Eq, t0 : Std::LessThanOrEq, t1 : Std::Eq, t1 : Std::LessThanOrEq] (t0, t1) : Std::LessThanOrEq`

### `impl [t0 : Std::ToString, t1 : Std::ToString] (t0, t1) : Std::ToString`

### `impl [t0 : Std::Eq, t1 : Std::Eq, t2 : Std::Eq] (t0, t1, t2) : Std::Eq`

### `impl [t0 : Std::Eq, t0 : Std::LessThan, t1 : Std::Eq, t1 : Std::LessThan, t2 : Std::Eq, t2 : Std::LessThan] (t0, t1, t2) : Std::LessThan`

### `impl [t0 : Std::Eq, t0 : Std::LessThanOrEq, t1 : Std::Eq, t1 : Std::LessThanOrEq, t2 : Std::Eq, t2 : Std::LessThanOrEq] (t0, t1, t2) : Std::LessThanOrEq`

### `impl [t0 : Std::ToString, t1 : Std::ToString, t2 : Std::ToString] (t0, t1, t2) : Std::ToString`

### `impl Std::Array : Std::Functor`

### `impl Std::Array : Std::Monad`

### `impl Std::Array a : Std::Add`

Concatenates two arrays.

### `impl Std::Array a : Std::Boxed`

### `impl [a : Std::Eq] Std::Array a : Std::Eq`

### `impl [a : Std::Eq, a : Std::LessThan] Std::Array a : Std::LessThan`

`LessThan` implementation for `Array a`.

Compares two arrays by lexicographic order.

### `impl [a : Std::Eq, a : Std::LessThanOrEq] Std::Array a : Std::LessThanOrEq`

`LessThanOrEq` implementation for `Array a`.

Compares two arrays by lexicographic order.

### `impl [a : Std::ToString] Std::Array a : Std::ToString`

### `impl Std::Array a : Std::Zero`

The empty array with zero capacity.

### `impl Std::Arrow a : Std::Functor`

### `impl Std::Arrow a : Std::Monad`

### `impl Std::Bool : Std::Eq`

### `impl Std::Bool : Std::Not`

### `impl Std::Bool : Std::ToString`

### `impl Std::Box a : Std::Boxed`

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

### `impl Std::FFI::Destructor a : Std::Boxed`

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

Concatenates two iterators.

### `impl [a : Std::Eq] Std::Iterator a : Std::Eq`

### `impl Std::Iterator a : Std::Zero`

The empty iterator.

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

### `impl Std::String : Std::Zero`

The empty string.

### `impl Std::Tuple2 t0 : Std::Functor`

### `impl Std::Tuple3 t0 t1 : Std::Functor`

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

### `loop : s -> (s -> Std::LoopState s r) -> r`

`loop` enables you to make a loop. `LoopState` is a union type defined as follows:

```
type LoopState s r = unbox union { continue : s, break : r };
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

### `loop_m : [m : Std::Monad] s -> (s -> m (Std::LoopState s r)) -> m r`

Monadic loop function. This is similar to `loop` but can be used to perform monadic action at each loop.

It is convenient to use `continue_m` and `break_m` to create monadic loop body function.

The following program prints "Hello World! (i)" for i = 0, 1, 2.

```
module Main;

main : IO ();
main = (
    loop_m(0, |i| (
        if i == 3 { break_m $ () };
        println("Hello World! (" + i.to_string + ")");;
        continue_m $ i + 1
    ))
);
```

### `mark_threaded : a -> a`

Traverses all values reachable from the given value, and changes the reference counters of them into multi-threaded mode.

### `undefined : Std::String -> a`

Generates an undefined value.

Calling this function prints `msg` to the stderr, flush stderr, and aborts the program (calls `abort` in libc).
Since `undefined(msg)` has generic type `a`, you can put it anywhere and it will be type-checked.

This is useful when you want to write a placeholder that will be implemented later:

```
truth : I64;
truth = undefined("I will implement the truth later.");
```

Another use case is aborting the program when a certain branch of the code should not be reached:

```
if condition {
    // Do something.
} else {
    undefined("This branch should not be reached.");
}
```

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
    assert_eq(|_|"fail: int_val is shared", unique, true);;

    // For boxed value, it returns true if the value isn't used later.
    let arr = Array::fill(10, 10);
    let (unique, arr) = arr.unsafe_is_unique;
    let use = arr.@(0); // This `arr` is not the one passed to `is_unique`, but the one returned by `is_unique`.
    assert_eq(|_|"fail: arr is shared", unique, true);;

    // Fox boxed value, it returns false if the value will be used later.
    let arr = Array::fill(10, 10);
    let (unique, _) = arr.unsafe_is_unique;
    let use = arr.@(0);
    assert_eq(|_|"fail: arr is unique", unique, false);;

    pure()
);
```

### `with_retained : (a -> b) -> a -> b`

`x.with_retained(f)` runs `f` with retained `x`. 
It is guaranteed that `x` is keep alive until `with_retained` is finished, even after `f` has finished using `x` in it. 

A typical use case of this function is the implementation of `Std::Array::borrow_ptr`.

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

@deprecated
Use `Std::FFI::_get_boxed_ptr` instead.

### `_get_sub_size_with_length_and_additional_capacity : Std::I64 -> Std::I64 -> Std::I64 -> Std::I64 -> Std::Array a -> Std::Array a`

A function like `get_sub`, but behaves as if the size of the array is the specified value,
and has a parameter to specify additional capacity of the returned `Array`.

### `_sort_range_using_buffer : Std::Array a -> Std::I64 -> Std::I64 -> ((a, a) -> Std::Bool) -> Std::Array a -> (Std::Array a, Std::Array a)`

Sorts elements in a range of a vector by "less than" comparator.

This function receives a working buffer as the first argument to reduce memory allocation, and returns it as second element.

### `_unsafe_force_unique : Std::Array a -> Std::Array a`

Force the uniqueness of an array.
If the given array is shared, this function returns the cloned array.

DEPRECATED:
This function is unsafe and deprecated because it is fragile when the "common expression elimination" optimization is implemented in the future. 
Consider the following example:

```
f : Array a -> Array a
f = |arr| arr.force_unique.do_something_for_unique_array;

let x = [1, 2, 3];
let y = f(x);
let z = f(x);
```

When this function `f` is inlined, the code will be as follows.

```
let x = [1, 2, 3];
let y = x.force_unique.do_something_for_unique_array;
let z = x.force_unique.do_something_for_unique_array;
```

Here, if the optimization is applied to the two `x.force_unique`, the code will call `do_something_for_unique_array` with a non-unique array.

```
let x = [1, 2, 3];
let x = x.force_unique;
let y = x.do_something_for_unique_array; // Here `x` is not unique
let z = x.do_something_for_unique_array;
```

Therefore, to use this function safely, you need to suppress the inlining of the above `f`. It is uncertain whether a function attribute such as "noinline" will be added in the future, so this function is deprecated currently.

### `_unsafe_get : Std::I64 -> Std::Array a -> a`

Gets a value from an array and returns it paired with the array itself, without bounds checking and retaining the value.

### `_unsafe_get_linear : Std::I64 -> Std::Array a -> (Std::Array a, a)`

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

### `empty : Std::I64 -> Std::Array a`

Creates an empty array with specified capacity.

### `fill : Std::I64 -> a -> Std::Array a`

Creates an array of the specified length filled with the initial value.

The capacity is set to the same value as the length.

Example: `fill(n, x) == [x, x, x, ..., x]` (of length `n`).

### `find_by : (a -> Std::Bool) -> Std::Array a -> Std::Option Std::I64`

Finds the first index at which the element satisfies a condition.

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

`arr.get_sub(s, e)` returns an array `[ arr.@(i) | i ∈ [s, e) ]`.

`s` and `e` are clamped to the range `[0, arr.get_size]`.

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

## `namespace Std::Box`

### `make : a -> Std::Box a`

## `namespace Std::Debug`

### `_debug_print_to_stream : Std::IO::IOHandle -> Std::String -> ()`

Prints a string to the specified stream and flushes the stream.

NOTE: This function is not pure and should only be used for temporary debugging purposes.

### `assert : (() -> Std::String) -> Std::Bool -> Std::IO ()`

Asserts that a condition (boolean value) is true.

If the assertion failed, prints a message to the stderr and aborts the program.

### `assert_eq : [a : Std::Eq] (() -> Std::String) -> a -> a -> Std::IO ()`

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

NOTE: This function is not pure and should only be used for temporary debugging purposes.

### `debug_eprint : Std::String -> ()`

Prints a string to stderr and flushes.

NOTE: This function is not pure and should only be used for temporary debugging purposes.

### `debug_eprintln : Std::String -> ()`

Prints a string followed by a newline to stderr and flushes.

NOTE: This function is not pure and should only be used for temporary debugging purposes.

### `debug_print : Std::String -> ()`

Prints a string to stdout and flushes.

NOTE: This function is not pure and should only be used for temporary debugging purposes.

### `debug_println : Std::String -> ()`

Prints a string followed by a newline to stdout and flushes.

NOTE: This function is not pure and should only be used for temporary debugging purposes.

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

### `_get_boxed_ptr : [a : Std::Boxed] a -> Std::Ptr`

Returns a pointer to the data of a boxed value.

The returned pointer points to:
- if the value is an `Array`, the first element of the array,
- if the value is a struct, the first field,
- if the value is an union, the data field (not the tag field).

The difference from `boxed_to_retained_ptr` is that this function returns a pointer to region where the payload of a boxed value is stored;
on the other hand, `boxed_to_retained_ptr` returns a pointer to the boxed value itself (which currently points to the reference counter of the boxed value).

NOTE: 
This function is unsafe in that if the call `v._get_boxed_ptr` is the last usage of `v`, then this function deallocates `v` and returns a dangling pointer.
To avoid this issue, use `borrow_boxed`, `borrow_boxed_io`, `mutate_boxed`, or `mutate_boxed_io` instead.

### `borrow_boxed : [a : Std::Boxed] (Std::Ptr -> b) -> a -> b`

Borrows a pointer to the data of a boxed value.

For the details of the pointer, see the document of `_get_boxed_ptr`.

### `borrow_boxed_io : [a : Std::Boxed] (Std::Ptr -> Std::IO b) -> a -> Std::IO b`

Performs an IO action borrowing a pointer to the data of a boxed value.

For the details of the pointer, see the document of `_get_boxed_ptr`.

### `boxed_from_retained_ptr : [a : Std::Boxed] Std::Ptr -> a`

Creates a boxed value from a retained pointer obtained by `boxed_to_retained_ptr`.

NOTE: 
It is the user's responsibility to ensure that the argument is actually a pointer to the type of the return value, and undefined behavior will occur if it is not.

### `boxed_to_retained_ptr : [a : Std::Boxed] a -> Std::Ptr`

Returns a retained pointer to a boxed value.
This function is used to share ownership of Fix's boxed values with foreign languages.

To get back the boxed value from the retained pointer, use `from_retained_ptr`.
To release / retain the value in a foreign language, call the function pointer obtained by `get_funptr_release` or `get_funptr_retain` on the pointer.

Note that the returned pointer points to the control block allocated by Fix, and does not necessary points to the data of the boxed value.
If you want to get a pointer to the data of the boxed value, use `borrow_boxed`.

### `clear_errno : Std::IO ()`

Sets errno to zero.

### `get_errno : Std::IO Std::I32`

Gets errno which is set by C functions.

### `get_funptr_release : [a : Std::Boxed] (() -> a) -> Std::Ptr`

Returns a pointer to the function of type `void (*)(void*)` which releases a boxed value of type `a`.
This function is used to release a pointer obtained by `boxed_to_retained_ptr`.

Note that this function is requires a value of type `Lazy a`, not of `a`.
So you can get release function for a boxed type `T` even when you don't have a value of type `T` -- you can just use `|_| undefined("") : T`:

```
module Main;

type VoidType = box struct {};
// No constructor for `VoidType` is provided.

main: IO ();
main = (
    let release = (|_| undefined("") : VoidType).get_funptr_release; // Release function of `VoidType`.
    pure()
);
```

### `get_funptr_retain : [a : Std::Boxed] (() -> a) -> Std::Ptr`

Returns a pointer to the function of type `void (*)(void*)` which retains a boxed value of type `a`.
This function is used to retain a pointer obtained by `boxed_to_retained_ptr`.

For the reason that this function requires a value of type `Lazy a`, not of `a`, see the document for `get_funptr_release`.

### `mutate_boxed : [a : Std::Boxed] (Std::Ptr -> Std::IO b) -> a -> (a, b)`

`x.mutate_boxed(io)` gets a pointer `ptr` to the data that `x` points to, executes `io(ptr)`, and then returns mutated `x` paired with the result of ``io(ptr)``.

The IO action `io(ptr)` is expected to modify the value of `x` through the obtained pointer. 
Do not perform any IO operations other than mutating the value of `x`.

For more details on the value of the pointer passed to `io`, see the document of `_get_boxed_ptr`.

This function first clones the value if `x` is not unique.

### `mutate_boxed_io : [a : Std::Boxed] (Std::Ptr -> Std::IO b) -> a -> Std::IO (a, b)`

`x.mutate_boxed_io(io)` gets a pointer `ptr` to the data that `x` points to, executes `io(ptr)`, and then returns mutated `x` paired with the result of `io(ptr)`.

Similar to `mutate_boxed`, but this function is used when you want to run the IO action in the existing IO context.

For more details, see the document of `mutate_boxed`.

### `mutate_boxed_ios : [a : Std::Boxed] (Std::Ptr -> Std::IO b) -> a -> Std::IO::IOState -> (Std::IO::IOState, (a, b))`

Internal implementation of the `mutate_boxed_io` function.

## `namespace Std::FFI::Destructor`

### `borrow : (a -> b) -> Std::FFI::Destructor a -> b`

Borrow the contained value.

`borrow(worker, dtor)` calls `worker` on the contained value captured by `dtor`, and returns the value returned by `worker`.

It is guaranteed that the `dtor` is alive during the call of `worker`.
In other words, the `worker` receives the contained value for which the destructor is not called yet.

### `borrow_io : (a -> Std::IO b) -> Std::FFI::Destructor a -> Std::IO b`

Performs an IO action borrowing the contained value.

### `make : a -> (a -> Std::IO a) -> Std::FFI::Destructor a`

Make a destructor value.

### `mutate_unique : (a -> Std::IO a) -> (a -> Std::IO b) -> Std::FFI::Destructor a -> (Std::FFI::Destructor a, b)`

Apply an IO action which mutates the semantics of the value.

`dtor.mutate_unique(ctor, action)` applies `action` to `dtor` if `dtor` is unique.
If `dtor` is shared, it creates a new `Destructor` value using `ctor` and applies `action` to the new value.

The `action` is allowed to modify the external resource stored in `dtor` (e.g., if `value` is a pointer, it can modify the value pointed by the pointer).
Also, `ctor` should be a "copy constructor" (e.g., memcpy) of the external resource stored in `dtor`.

### `mutate_unique_io : (a -> Std::IO a) -> (a -> Std::IO b) -> Std::FFI::Destructor a -> Std::IO (Std::FFI::Destructor a, b)`

Apply an IO action which mutates the semantics of the value.

This is similar to `mutate_unique`, but the `ctor` and `action` is executed in the context of the external `IO` context.

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

### `_read_line_inner : Std::Bool -> Std::IO::IOHandle -> Std::IO::IOFail Std::String`

Reads characters from an IOHandle.

If the first argument `upto_newline` is true, this function reads a file upto newline or EOF.

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

### `from_runner : (Std::IO::IOState -> (Std::IO::IOState, a)) -> Std::IO a`

Creates an IO action from a IO runner function, which is a function of type `IOState -> (IOState, a)`.

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

### `loop_lines : Std::IO::IOHandle -> s -> (s -> Std::String -> Std::LoopState s s) -> Std::IO::IOFail s`

Loop on lines read from an `IOHandle`.

`loop_lines(handle, initial_state, worker)` calls `worker` on the pair of current state and a line string read from `handle`.
The function `worker` should return an updated state as `LoopState` value, i.e., a value created by `continue` or `break`.
When the `handle` reaches to the EOF or `worker` returns a `break` value, `loop_lines` returns the last state value.

Note that the line string passed to `worker` may contain a newline code at the end. To remove it, use `String::strip_last_spaces`.

### `loop_lines_io : Std::IO::IOHandle -> s -> (s -> Std::String -> Std::IO::IOFail (Std::LoopState s s)) -> Std::IO::IOFail s`

Loop on lines read from an `IOHandle`.

Similar to `loop_lines`, but the worker function can perform an IO action.

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

### `stderr : Std::IO::IOHandle`

The handle for standard error.

### `stdin : Std::IO::IOHandle`

The handle for standard input.

### `stdout : Std::IO::IOHandle`

The handle for standard output.

### `unsafe_perform : Std::IO a -> a`

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

### `from_io_result : Std::IO (Std::Result Std::String a) -> Std::IO::IOFail a`

Create from IO action of which returns `Result ErrMsg a`.

### `from_result : Std::Result Std::String a -> Std::IO::IOFail a`

Creates an pure `IOFail` value from a `Result` value.

### `lift : Std::IO a -> Std::IO::IOFail a`

Lifts an `IO` action to a successful `IOFail` action.

### `throw : Std::String -> Std::IO::IOFail a`

Creates an error `IOFail` action.

### `to_result : Std::IO::IOFail a -> Std::IO (Std::Result Std::String a)`

Converts an `IOFail` to an `Result` value (wrapped by `IO`).

### `try : (Std::String -> Std::IO a) -> Std::IO::IOFail a -> Std::IO a`

Converts an `IOFail` value to an `IO` value by an error handler (i.e., a `catch`) function.

## `namespace Std::IO::IOHandle`

### `_file_ptr : Std::IO::IOHandle -> Std::Ptr`

Gets pointer to C's `FILE` value from an `IOHandle`.

If the `IOHandle` is already closed, the function returns `nullptr`.

NOTE:
Do not directly close the file pointer by `fclose` or other functions.
Instead you should close `IOHandle` by `IO::close_file`.

DEPRECATED:
Use `get_file_ptr` instead.
This function is deprecated because it has a pure function interface, but the value of `_file_ptr` changes by calling `IO::close_file`.

### `from_file_ptr : Std::Ptr -> Std::IO::IOHandle`

Creates an `IOHandle` from a file pointer (i.e., pointer to C's `FILE`).

Creating two `IOHandle`s from a single file pointer is forbidden.

### `get_file_ptr : Std::IO::IOHandle -> Std::IO Std::Ptr`

Gets pointer to C's `FILE` value from an `IOHandle`.

If the `IOHandle` is already closed, the function returns `nullptr`.

NOTE:
Do not directly close the file pointer by `fclose` or other functions.
Instead you should close `IOHandle` by `IO::close_file`.

NOTE:
If `IO::close` is called while using the `Ptr` obtained by this function, the `Ptr` becomes invalid and may cause undefined behavior.

## `namespace Std::Iterator`

### `_flatten : Std::Iterator (Std::Iterator a) -> Std::Iterator a`

Flatten an iterator of iterators.

You should use `Monad::flatten` instead of this function.
This function is used in the implementation of `Monad::bind` for `Iterator`.

### `_flatten_sub : Std::Iterator a -> Std::Iterator (Std::Iterator a) -> Std::Iterator a`

### `advance : Std::Iterator a -> Std::Option (a, Std::Iterator a)`

Gets next value and next iterator.

### `append : Std::Iterator a -> Std::Iterator a -> Std::Iterator a`

Appends an iterator to a iterator.

Note: Since `iter1.append(iter2)` puts `iter2` after `iter1`, `append(lhs, rhs)` puts `lhs` after `rhs`.

### `bang : Std::Iterator a -> Std::Iterator a`

Evaluates all elements of iterator.

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

`[a0, a1, a2, ...].fold(s, op) = ...op(op(op(s, a0), a1), a2)...`

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

### `loop_iter : s -> (s -> a -> Std::LoopState s s) -> Std::Iterator a -> s`

Loop along an iterator.

Unlike `fold`, you can break the loop by returning `break` at each iteration step.

### `loop_iter_m : [m : Std::Monad] s -> (s -> a -> m (Std::LoopState s s)) -> Std::Iterator a -> m s`

Loop by monadic action along an iterator.

Unlike `fold_m`, you can break the loop by returning `break_m` at each iteration step.

### `product : Std::Iterator a -> Std::Iterator b -> Std::Iterator (b, a)`

Generates the cartesian product of two iterators.

Example: `[1, 2, 3].to_iter.product(['a', 'b'].to_iter).to_array == [(1, 'a'), (2, 'a'), (3, 'a'), (1, 'b'), (2, 'b'), (3, 'b')]`

### `push_front : a -> Std::Iterator a -> Std::Iterator a`

Pushes an element to an iterator.

### `range : Std::I64 -> Std::I64 -> Std::Iterator Std::I64`

Creates a range, i.e. an iterator of the form `[a, a+1, a+2, ..., b-1]`.

### `reverse : Std::Iterator a -> Std::Iterator a`

Reverses an iterator.

### `subsequences : Std::Iterator a -> Std::Iterator (Std::Iterator a)`

Generates all subsequences of an iterator.

`[1,2,3].to_iter.subsequences` is `[[], [3], [2], [2, 3], [1], [1, 3], [1, 2], [1, 2, 3]].to_iter.map(to_iter)`.

### `sum : [a : Std::Additive] Std::Iterator a -> a`

Calculates the sum of elements of an iterator.

### `take : Std::I64 -> Std::Iterator a -> Std::Iterator a`

Takes at most n elements from an iterator.

### `take_while : (a -> Std::Bool) -> Std::Iterator a -> Std::Iterator a`

Takes elements of an iterator while a condition is satisfied.
TODO: add test

### `to_array : Std::Iterator a -> Std::Array a`

Converts an iterator to an array.

### `zip : Std::Iterator b -> Std::Iterator a -> Std::Iterator (a, b)`

Zips two iterators.

## `namespace Std::LessThan`

### `less_than : [a : Std::LessThan] a -> a -> Std::Bool`

Compares two values. An expression `x < y` is translated to `less_than(x, y)`.

### `max : [a : Std::LessThan] a -> a -> a`

### `min : [a : Std::LessThan] a -> a -> a`

## `namespace Std::LessThanOrEq`

### `less_than_or_eq : [a : Std::LessThanOrEq] a -> a -> Std::Bool`

Compares two values. An expression `x <= y` is translated to `less_than_or_eq(x, y)`.

## `namespace Std::LoopState`

### `break_m : [m : Std::Monad] r -> m (Std::LoopState s r)`

Make a break value wrapped in a monad.

This is used with `loop_m` function.

### `continue_m : [m : Std::Monad] s -> m (Std::LoopState s r)`

Make a continue value wrapped in a monad.

This is used with `loop_m` function.

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

### `as_some_or : a -> Std::Option a -> a`

Unwrap an option value if it is `some`, or returns given default value if it is `none`.

### `map_or : b -> (a -> b) -> Std::Option a -> b`

Returns the provided default value if the option is none, or applies a function to the contained value if the option is some.

## `namespace Std::Path`

### `parse : Std::String -> Std::Option Std::Path`

Parse a string.

## `namespace Std::Ptr`

### `add_offset : Std::I64 -> Std::Ptr -> Std::Ptr`

Adds an offset to a pointer.

### `subtract_ptr : Std::Ptr -> Std::Ptr -> Std::I64`

Subtracts two pointers.

Note that `x.subtract_ptr(y)` calculates `x - y`, so `subtract_ptr(x, y)` calculates `y - x`.

## `namespace Std::PunchedArray`

### `plug_in : a -> Std::PunchedArray a -> Std::Array a`

Plug in an element to a punched array to get back an array.

### `unsafe_punch : Std::I64 -> Std::Array a -> (Std::PunchedArray a, a)`

Creates a punched array by moving out the element at the specified index.

NOTE: this function assumes that the given array is unique WITHOUT CHECKING.
The uniqueness of the array is ensured in the `Array::act` function.

## `namespace Std::Rem`

### `rem : [a : Std::Rem] a -> a -> a`

Calculate remainder of a value dividing another value. An expression `x % y` is translated to `rem(x, y)`.

## `namespace Std::Result`

### `unwrap : Std::Result e o -> o`

Returns the containing value if the value is ok, or otherwise aborts the program.

## `namespace Std::String`

### `_get_c_str : Std::String -> Std::Ptr`

Get the null-terminated C string.

Note that in case the string is not used after call of this function, the returned pointer will be already released.

### `_unsafe_from_c_str : Std::Array Std::U8 -> Std::String`

Create a string from C string (i.e., null-terminated byte array).

If the byte array doesn't include `\0`, this function causes undefined behavior.

### `_unsafe_from_c_str_ptr : Std::Ptr -> Std::String`

Create a `String` from a pointer to null-terminated C string.

If `ptr` is not pointing to a valid null-terminated C string, this function cause undefined behavior.

### `borrow_c_str : (Std::Ptr -> a) -> Std::String -> a`

Call a function with a null-terminated C string.

### `borrow_c_str_io : (Std::Ptr -> Std::IO a) -> Std::String -> Std::IO a`

Call an IO action with a null-terminated C string.

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

### `from_U8 : Std::U8 -> Std::String`

Creates a string from a byte.

Example:
```
assert_eq(|_|"", String::from_U8('a'), "a");;
assert_eq(|_|"", String::from_U8('\x00'), "");;
```

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

### `pop_back_byte : Std::String -> Std::String`

Removes the last byte.

If the string is empty, this function does nothing.

### `split : Std::String -> Std::String -> Std::Iterator Std::String`

`str.split(sep)` splits `str` by `sep` into an iterator.

Example:
```
assert_eq(|_|"Ex. 1", "ab,c,".split(",").to_array, ["ab", "c", ""]);;
assert_eq(|_|"Ex. 2", "abc".split(",").to_array, ["abc"]);;
assert_eq(|_|"Ex. 3", "abc".split("").to_array, ["a", "b", "c"]);; // Special behavior when the separator is empty.
```

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