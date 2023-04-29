# Table of contents

- [Table of contents](#table-of-contents)
- [Tutorial](#tutorial)
  - [An example program](#an-example-program)
  - [Modules](#modules)
  - [Global values](#global-values)
  - [Namespaces](#namespaces)
  - [Types](#types)
  - [Expressions](#expressions)
  - [Let-expressions](#let-expressions)
  - [If-expressions](#if-expressions)
  - [Function application](#function-application)
  - [Function definition](#function-definition)
  - [Operator `.` and `$`](#operator--and-)
  - [Patterns](#patterns)
  - [`loop`, `continue` and `break` function](#loop-continue-and-break-function)
  - [Unions](#unions)
  - [Structs](#structs)
  - [Iterators](#iterators)
  - [Mutation in Fix](#mutation-in-fix)
- [Other topics on syntax](#other-topics-on-syntax)
  - [Module and imports](#module-and-imports)
  - [Recursion](#recursion)
  - [Overloading](#overloading)
  - [Trait](#trait)
  - [Monad](#monad)
    - [What is monad?](#what-is-monad)
      - [State-like monads](#state-like-monads)
      - [Result-like monads](#result-like-monads)
      - [List-like monads](#list-like-monads)
    - [`do` block and monadic bind operator `*`](#do-block-and-monadic-bind-operator-)
  - [Type annotation](#type-annotation)
  - [Boxed and unboxed types](#boxed-and-unboxed-types)
    - [Functions](#functions)
    - [Tuples](#tuples)
    - [Unit](#unit)
    - [Array](#array)
    - [Structs](#structs-1)
    - [Unions](#unions-1)
  - [Calling C functions](#calling-c-functions)
- [Built-in / library features](#built-in--library-features)
  - [Types](#types-1)
    - [Structs](#structs-2)
      - [`@{field_name} : {struct} -> {field_type}`](#field_name--struct---field_type)
      - [`={field_name} : {field_type} -> {struct} -> {struct}`](#field_name--field_type---struct---struct)
      - [`={field_name}! : {field_type} -> {struct} -> {struct}`](#field_name--field_type---struct---struct-1)
      - [`mod_{field_name} : ({field_type} -> {field_type}) -> {struct} -> {struct}`](#mod_field_name--field_type---field_type---struct---struct)
      - [`mod_{field_name}! : ({field_type} -> {field_type}) -> {struct} -> {struct}`](#mod_field_name--field_type---field_type---struct---struct-1)
    - [Unions](#unions-2)
      - [`{variant_name} : {variant_type} -> {union}`](#variant_name--variant_type---union)
      - [`is_{variant_name} : {union} -> Bool`](#is_variant_name--union---bool)
      - [`as_{variant_name} : {union} -> {variant_type}`](#as_variant_name--union---variant_type)
      - [`mod_{variant_name} : ({variant_type} -> {variant_type}) -> {union} -> {union}`](#mod_variant_name--variant_type---variant_type---union---union)
    - [Std::Array](#stdarray)
      - [`__unsafe_set_length : I64 -> Array a -> Array a`](#__unsafe_set_length--i64---array-a---array-a)
      - [`__unsafe_get : I64 -> Array a -> a`](#__unsafe_get--i64---array-a---a)
      - [`__unsafe_set : I64 -> a -> Array a -> Array a`](#__unsafe_set--i64---a---array-a---array-a)
      - [`_get_ptr : Array a -> Ptr`](#_get_ptr--array-a---ptr)
      - [`_sort_range_using_buffer : Array a -> I64 -> I64 -> ((a, a) -> Bool) -> Array a -> (Array a, Array a)`](#_sort_range_using_buffer--array-a---i64---i64---a-a---bool---array-a---array-a-array-a)
      - [`append : Array a -> Array a -> Array a`](#append--array-a---array-a---array-a)
      - [`call_with_ptr : (Ptr -> b) -> Array a -> b`](#call_with_ptr--ptr---b---array-a---b)
      - [`empty : I64 -> Array a`](#empty--i64---array-a)
      - [`fill : I64 -> a -> Array a`](#fill--i64---a---array-a)
      - [`force_unique : Array a -> Array a`](#force_unique--array-a---array-a)
      - [`force_unique! : Array a -> Array a`](#force_unique--array-a---array-a-1)
      - [`from_map : I64 -> (I64 -> a) -> Array a`](#from_map--i64---i64---a---array-a)
      - [`get : I64 -> Array a -> a`](#get--i64---array-a---a)
      - [`get_first : Array a -> Option a`](#get_first--array-a---option-a)
      - [`get_last : Array a -> Option a`](#get_last--array-a---option-a)
      - [`get_length : Array a -> I64`](#get_length--array-a---i64)
      - [`get_capacity : Array a -> I64`](#get_capacity--array-a---i64)
      - [`is_empty : Array a -> Bool`](#is_empty--array-a---bool)
      - [`mod : I64 -> (a -> a) -> Array a -> Array a`](#mod--i64---a---a---array-a---array-a)
      - [`mod! : I64 -> (a -> a) -> Array a -> Array a`](#mod--i64---a---a---array-a---array-a-1)
      - [`pop_back : Array a -> Array a`](#pop_back--array-a---array-a)
      - [`push_back : a -> Array a -> Array a`](#push_back--a---array-a---array-a)
      - [`range : I64 -> I64 -> Iterator I64`](#range--i64---i64---iterator-i64)
      - [`reduce_length : I64 -> Array a -> Array a`](#reduce_length--i64---array-a---array-a)
      - [`reserve : I64 -> Array a -> Array a`](#reserve--i64---array-a---array-a)
      - [`set : I64 -> a -> Array a -> Array a`](#set--i64---a---array-a---array-a)
      - [`set! : I64 -> a -> Array a -> Array a`](#set--i64---a---array-a---array-a-1)
      - [`sort_by : ((a, a) -> Bool) -> Array a -> Array a`](#sort_by--a-a---bool---array-a---array-a)
    - [Std::Bool](#stdbool)
      - [`impl Bool : Eq`](#impl-bool--eq)
      - [`impl Bool : ToString`](#impl-bool--tostring)
    - [Std::F32](#stdf32)
      - [`abs : F32 -> F32`](#abs--f32---f32)
      - [`impl F32 : Add`](#impl-f32--add)
      - [`impl F32 : Div`](#impl-f32--div)
      - [`impl F32 : Eq`](#impl-f32--eq)
      - [`impl F32 : LessThan`](#impl-f32--lessthan)
      - [`impl F32 : LessThanOrEq`](#impl-f32--lessthanoreq)
      - [`impl F32 : Mul`](#impl-f32--mul)
      - [`impl F32 : Sub`](#impl-f32--sub)
      - [`impl F32 : ToF32`](#impl-f32--tof32)
      - [`impl F32 : ToF64`](#impl-f32--tof64)
      - [`impl F32 : ToString`](#impl-f32--tostring)
    - [Std::F64](#stdf64)
      - [`abs : F64 -> F64`](#abs--f64---f64)
      - [`impl F64 : Add`](#impl-f64--add)
      - [`impl F64 : Div`](#impl-f64--div)
      - [`impl F64 : Eq`](#impl-f64--eq)
      - [`impl F64 : LessThan`](#impl-f64--lessthan)
      - [`impl F64 : LessThanOrEq`](#impl-f64--lessthanoreq)
      - [`impl F64 : Mul`](#impl-f64--mul)
      - [`impl F64 : Sub`](#impl-f64--sub)
      - [`impl F64 : ToF32`](#impl-f64--tof32)
      - [`impl F64 : ToF64`](#impl-f64--tof64)
      - [`impl F64 : ToString`](#impl-f64--tostring)
    - [Std::IO](#stdio)
      - [`__unsafe_perform : IO a -> a`](#__unsafe_perform--io-a---a)
      - [`close_file : IOHandle -> IO ()`](#close_file--iohandle---io-)
      - [`open_file : Path -> String -> IOResult IOError IOHandle`](#open_file--path---string---ioresult-ioerror-iohandle)
      - [`print : String -> IO ()`](#print--string---io-)
      - [`println : String -> IO ()`](#println--string---io-)
      - [`read_content : IOHandle -> IOResult IOError String`](#read_content--iohandle---ioresult-ioerror-string)
      - [`read_file : Path -> IOResult IOError String`](#read_file--path---ioresult-ioerror-string)
      - [`read_line : IOHandle -> IOResult IOError String`](#read_line--iohandle---ioresult-ioerror-string)
      - [`read_line_inner : Bool -> IOHandle -> IOResult IOError String`](#read_line_inner--bool---iohandle---ioresult-ioerror-string)
      - [`with_file : Path -> String -> (IOHandle -> IOResult IOError a) -> IOResult IOError a`](#with_file--path---string---iohandle---ioresult-ioerror-a---ioresult-ioerror-a)
      - [`write_content : IOHandle -> String -> IOResult IOError ()`](#write_content--iohandle---string---ioresult-ioerror-)
      - [`write_file : Path -> String -> IOResult IOError ()`](#write_file--path---string---ioresult-ioerror-)
      - [`impl IO : Functor`](#impl-io--functor)
      - [`impl IO : Monad`](#impl-io--monad)
    - [Std::IO::IOError](#stdioioerror)
      - [`impl IOError : ToString`](#impl-ioerror--tostring)
    - [Std::IO::IOHandle](#stdioiohandle)
      - [`stderr : IOHandle`](#stderr--iohandle)
      - [`stdin : IOHandle`](#stdin--iohandle)
      - [`stdout : IOHandle`](#stdout--iohandle)
    - [Std::IO::IOResult](#stdioioresult)
      - [`from_result : Result e a -> IOResult e a`](#from_result--result-e-a---ioresult-e-a)
      - [`lift : IO a -> IOResult e a`](#lift--io-a---ioresult-e-a)
      - [`to_io : IOResult e a -> IO (Result e a)`](#to_io--ioresult-e-a---io-result-e-a)
      - [`impl IOResult e : Functor`](#impl-ioresult-e--functor)
      - [`impl IOResult e : Monad`](#impl-ioresult-e--monad)
    - [Std::I32](#stdi32)
      - [\_I32\_to\_string : I32 -\> String](#_i32_to_string--i32---string)
    - [Std::I64](#stdi64)
      - [\_I64\_to\_string : I64 -\> String](#_i64_to_string--i64---string)
    - [Std::Iterator](#stditerator)
      - [`advance : Iterator a -> Option (a, Iterator a)`](#advance--iterator-a---option-a-iterator-a)
      - [`append : Iterator a -> Iterator a -> Iterator a`](#append--iterator-a---iterator-a---iterator-a)
      - [`count_up : I64 -> Iterator I64`](#count_up--i64---iterator-i64)
      - [`empty : Iterator a`](#empty--iterator-a)
      - [`get_length : Iterator a -> I64`](#get_length--iterator-a---i64)
      - [`intersperse : a -> Iterator a -> Iterator a`](#intersperse--a---iterator-a---iterator-a)
      - [`is_empty : Iterator a -> Bool`](#is_empty--iterator-a---bool)
      - [`filter : (a -> Bool) -> Iterator a -> Iterator a`](#filter--a---bool---iterator-a---iterator-a)
      - [`fold : b -> (b -> a -> b) -> Iterator a -> b`](#fold--b---b---a---b---iterator-a---b)
      - [`from_array : Array a -> Iterator a`](#from_array--array-a---iterator-a)
      - [`from_map : (I64 -> a) -> Iterator a`](#from_map--i64---a---iterator-a)
      - [`push_front : a -> Iterator a -> Iterator a`](#push_front--a---iterator-a---iterator-a)
      - [`reverse : Iterator a -> Iterator a`](#reverse--iterator-a---iterator-a)
      - [`take : I64 -> Iterator a -> Iterator a`](#take--i64---iterator-a---iterator-a)
      - [`zip : Iterator a -> Iterator b -> Iterator (a, b)`](#zip--iterator-a---iterator-b---iterator-a-b)
      - [`impl Iterator a : Add`](#impl-iterator-a--add)
      - [`impl [a : Eq] Iterator a : Eq`](#impl-a--eq-iterator-a--eq)
      - [`impl Iterator : Functor`](#impl-iterator--functor)
      - [`impl Iterator : Monad`](#impl-iterator--monad)
    - [Std::Option](#stdoption)
      - [`impl [a : Eq] Option a : Eq`](#impl-a--eq-option-a--eq)
      - [`impl Option : Functor`](#impl-option--functor)
      - [`impl Option : Monad`](#impl-option--monad)
    - [Std::Path](#stdpath)
      - [`parse : String -> Option Path`](#parse--string---option-path)
    - [Std::Ptr](#stdptr)
    - [Std::Result](#stdresult)
      - [`unwrap : [e : ToString] Result e o -> o`](#unwrap--e--tostring-result-e-o---o)
      - [`impl Result e : Monad`](#impl-result-e--monad)
    - [Std::String](#stdstring)
      - [`_get_c_str : String -> Ptr`](#_get_c_str--string---ptr)
      - [`call_with_c_str : (Ptr -> a) -> String -> a`](#call_with_c_str--ptr---a---string---a)
      - [`concat : String -> String -> String`](#concat--string---string---string)
      - [`concat_iter : Iterator String -> String`](#concat_iter--iterator-string---string)
      - [`get_first_byte : String -> Option Byte`](#get_first_byte--string---option-byte)
      - [`get_last_byte : String -> Option Byte`](#get_last_byte--string---option-byte)
      - [`get_length : String -> I64`](#get_length--string---i64)
      - [`is_empty : String -> Bool`](#is_empty--string---bool)
      - [`join : String -> Iterator String -> String`](#join--string---iterator-string---string)
      - [`pop_back_byte : String -> String`](#pop_back_byte--string---string)
      - [`strip_last_bytes : (Byte -> Bool) -> String -> String`](#strip_last_bytes--byte---bool---string---string)
      - [`strip_last_newlines : String -> String`](#strip_last_newlines--string---string)
    - [Std::U8](#stdu8)
      - [\_U8\_to\_string : U8 -\> String](#_u8_to_string--u8---string)
    - [Std::U32](#stdu32)
      - [\_U32\_to\_string : U32 -\> String](#_u32_to_string--u32---string)
    - [Std::U64](#stdu64)
      - [\_U64\_to\_string : U64 -\> String](#_u64_to_string--u64---string)
  - [Functions](#functions-1)
    - [Std::is\_unique : a -\> (Bool, a)](#stdis_unique--a---bool-a)
    - [Std::fix : ((a -\> b) -\> a -\> b) -\> a -\> b](#stdfix--a---b---a---b---a---b)
    - [Std::loop : s -\> (s -\> LoopResult s r) -\> r](#stdloop--s---s---loopresult-s-r---r)
    - [Std::Debug::debug\_print : String -\> ()](#stddebugdebug_print--string---)
    - [Std::Debug::debug\_println : String -\> ()](#stddebugdebug_println--string---)
    - [Std::Debug::abort : () -\> a](#stddebugabort-----a)
    - [Std::Debug::assert : String -\> Bool -\> ()](#stddebugassert--string---bool---)
    - [Std::Debug::assert\_eq : \[a: Eq\] String -\> a -\> a -\> ()](#stddebugassert_eq--a-eq-string---a---a---)
  - [Traits](#traits)
    - [Std::Functor (\* -\> \*)](#stdfunctor----)
      - [`map : [f : Functor] (a -> b) -> f a -> f b`](#map--f--functor-a---b---f-a---f-b)
    - [Std::Monad (\* -\> \*)](#stdmonad----)
      - [(required) `bind : [m : Monad] (a -> m b) -> m a -> m b`](#required-bind--m--monad-a---m-b---m-a---m-b)
      - [`flatten : [m : Monad] m (m a) -> a`](#flatten--m--monad-m-m-a---a)
      - [(required) `pure : [m : Monad] a -> m a`](#required-pure--m--monad-a---m-a)
    - [Std::ToString](#stdtostring)
      - [`to_string : [a: ToString] a -> String`](#to_string--a-tostring-a---string)
    - [Std::ToI32](#stdtoi32)
      - [`to_I32 : [a: ToI32] a -> I32`](#to_i32--a-toi32-a---i32)
    - [Std::ToI64](#stdtoi64)
      - [`to_I64 : [a: ToI64] a -> I64`](#to_i64--a-toi64-a---i64)
    - [Std::ToU8](#stdtou8)
      - [`to_U8 : [a: ToU8] a -> U8`](#to_u8--a-tou8-a---u8)
    - [Std::ToU32](#stdtou32)
      - [`to_U32 : [a: ToU32] a -> U32`](#to_u32--a-tou32-a---u32)
    - [Std::ToU64](#stdtou64)
      - [`to_U64 : [a: ToU64] a -> U64`](#to_u64--a-tou64-a---u64)
  - [Operators](#operators)


# Tutorial

## An example program

The following is a Fix program that calculates the first 30 numbers of Fibonacci sequence. 

```
module Main;

calc_fib : I64 -> Array I64;
calc_fib = |n| (
    let arr = Array::fill(n, 0);
    let arr = arr.set!(0, 1);
    let arr = arr.set!(1, 1);
    let arr = loop((2, arr), |(idx, arr)|
        if idx == arr.get_length {
            break $ arr
        } else {
            let x = arr.get(idx-1);
            let y = arr.get(idx-2);
            let arr = arr.set!(idx, x+y);
            continue $ (idx+1, arr)
        }
    );
    arr
);

main : IO ();
main = (
    let fib = calc_fib(30);
    println $ Iterator::from_array(fib).map(to_string).join(", ")
);
```

If you save the above program to a file "main.fix" and run `fix run main.fix`, it prints 

```
1, 1, 2, 3, 5, 8, 13, 21, 34, 55, 89, 144, 233, 377, 610, 987, 1597, 2584, 4181, 6765, 10946, 17711, 28657, 46368, 75025, 121393, 196418, 317811, 514229, 832040
```

to the standard output.

In the followings, I explain language specifications which are necessary to understand the above program.

## Modules

The first line is the module definition:

```
module Main;
```

In Fix, values, functions, types and traits defined in a source file is collected to a module. Each source file has to declare the name of the module it defines by `module {module_name};`. The first letter of the module name must be capitalized.

When Fix program runs, it calls `main` function defined in the `Main` module.

The usefulness of modules is hard to see in this example. They are useful when you construct a program from multiple source files.

## Global values

The following parts are definitions of two global values `calc_fib` and `main`.

```
calc_fib : I64 -> Array I64;
calc_fib = ...{expression A}...;

main : IO ();
main = ...{expression B}...;
```

These lines means that:

- `calc_fib` global value has type `I64 -> Array I64` and it's value is defined by expression A.
- `main` global value has type `IO ()` and it's value is defined by expression B.

In Fix, you have to specify the type of a global value explicitly. 

## Namespaces

The `Array` in `Array::fill` or `Iterator` in `Iterator::from_array` are namespaces. Namespace is the "address" of a name and used to distinguish two values (or types or traits, anything you define globally) with the same name.

Namespaces of a name can be omitted if the value specified by the name is unique, or can be inferred from the context. In fact, you can write simply `fill(n, 0)` instead of `Array::fill(n, 0)` because there is only one function named `fill` at the current version of standard library. The reasons I wrote `Array::fill(n, 0)` here are:

- `Array::fill(n, 0)` is more readable than `fill(n, 0)`, because it expresses that `fill` function is related to `Array` type. A reader may be able to infer that `Array::fill` will generate an array of specified length filled by a specified initial value.
- In the future, another function named `fill` may be added to a namespace other than `Array`. After that, the name `fill` may become ambiguous and the compile of the example program may start to fail.

Actually, the full name of `fill` is not `Array::fill` but `Std::Array::fill`. `Std` is a module to put values standard library provides. Module is nothing but a top-level namespace. The namespace `Array` is defined as the sub-namespace of `Std` and used to put functions related to arrays. Similarly, full name of `calc_fib` function is `Main::calc_fib`. You can omit (possibly full) prefix of namespaces of a name as long as the value referred to is uniquely inferred by compiler from the context.

## Types

Each value in Fix has it's type. You can consider that a type is a set in mathematics, and value in Fix is an element of it's type. 

The followings are examples of types:

- `I64`: the type of 64-bit signed integers.
- `Bool`: the type of boolean values (i.e., `true` and `false`).
- `Array a`: the type of arrays whose elements have type `a`. `Array` is called a type constructor, because it generates types `Array I64` or `Array Bool` when applied to a type. `a` is called a type parameter.
- `String`: the type of strings.
- `I64 -> Array I64`: the type of functions that takes an integer and returns an array of integers.
- `()`: the unit type. This type has a single value which is also written as `()`. 
- `(a, b)`: the type of pairs of values of `a` and `b`, where `a` and `b` are type parameters.
- `IO a`: the type whose value corresponds to an I/O action such as printing a string, opening a file and reading it's content, etc. The type variable `a` is for the type of values returned by the I/O action. For example, if an I/O action reads the standard input as a `String` (and if we assume it never fails), it should have type `IO String`.
- `IO ()`: the type of I/O actions which returns no value. It is the type of `main` function of Fix program.
- `I64 -> Bool -> Array Bool`: this is equivalent to `I64 -> (Bool -> Array Bool)`, that is, the type of functions that receives an integer and returns a function that converts a boolean value into a boolean array. As an example, a function that produces a boolean array from it's length and initial value has this type. In Fix, there is no concept of "two-variable functions". A function in Fix is a (partial) function in mathematical sense: it converts an element of a set into an element of another set (or fails). The type of something like "two-variable functions" can be represented as `a -> b -> c` or `(a, b) -> c`.

In Fix, the first letter of the name of a specific type (such as `I64` or `Bool`) or a type constructor (such as `Array`) has to be 
capitalized. A type that starts with a lowercase letter is interpreted as a type parameter. Each type parameter will be instanciated to a specific type when the program is compiled.

## Expressions

Expression is a sentence which describes a value. The followings are examples of expressions:

- `42`: a literal expression which means the number 42 represented as a signed 64-bit integer.
- `false`, `true`: literal expressions which means boolean value (represented as a 8-bit integer `0` and `1` internally).
- `[1, 2, 3]`: a literal expression which means an integer array with elements `1`, `2` and `3`.
- `"Hello World!"`: a string literal.
- `()`: the unit literal, whose type is also written as `()` and called "the unit type".
- `(1, true)`: a tuple literal, which produces a value of the type `(I64, Bool)`.
- `3 + 5`: an expression which means "the integer obtained by adding `3` and `5`".
- `let x = 3 + 5 in x * x`: an expression which means "Compute `3 + 5` and call the result `x`. Then compute `x * x`."
- `if c { x + y } else { x - y }`: an expression which means "If a boolean value `c` is `true`, then the value of this expression is `x + y`. Otherwise, the value of this expression is `x - y`".
- `f(x)`: an expression which means "the value obtained by applying a function `f` to the value `x`".
- `|x| x + 3`: an expression which means "the function which converts `x` to `x + 3`".

## Let-expressions

To define a local name by a value, use `let`-expression. The syntax is `let {name} = {expression_0} in {expression_1}` or `let {name} = {expression_0}; {expression_1}`.

If you write the whole let-expression in one line, it is preferred to use `in`: For example, `let x = 5 in 2 + x`. Of course, you can also write it as `let x = 5; 2 + x`.

On the other hand, if you want to put `{epxression_0}` and `{expression_1}` in other lines, it is better to use semicolon:
```
let x = 3;
let y = 5;
x + y
```

If `{expression_0}` ranges several lines, it is preferred to indent `{expression_0}` with parenthes. For example, the following expression:
```
let sixty_four = (
    let n = 3 + 5;
    n * n
);
sixty_four + sixty_four
```
which is evaluated to 128, can also be written as 
```
let sixty_four = 
let n = 3 + 5;
n * n;
sixty_four + sixty_four
```
because the indent and parenthes are not mandatory, but the latter is less readable and not recommended.

Fix's `let`-expression doesn't allow recursive definition. For example, a program

```
use_rec_defn : I64;
use_rec_defn = let x = x + 3 in x * x;
```

cannot be compiled. A program

```
use_rec_defn : I64;
use_rec_defn = (
    let x = 5;
    let x = x + 3;
    x * x
);
```

will be compiled, but the name `x` in the right hand side of `let x = x + 3` is considered as the name `x` defined in the previous line (i.e., it's value is `5`), not as the new one.

This means that you cannot define a local recursive function by let-expression naively. To do this, use `fix` built-in function.

## If-expressions

The syntax of `if` is the following: `if cond { expr_0 } (else|;) { expr_1 }` where curly braces around `expr_1` is optional.
The type of `cond` has to be `Bool`, and types of `expr_0` and `expr_1` must coincide.

For usual case, use `if cond { expr_0 } else { expr_1 }`:
```
if cond { 
    "cond is true!"
} else {
    "cond is false!"
}
```

To write "early return" pattern, it is useful to omit curly braces around `{expr_1}`:
```
if cache_is_available { "the cached value" };
"a long program which calculates a value, store it into cache, and returns the value."
```

## Function application

To apply a function `f` to a value `x`, write `f(x)`.

```
neg(3) // -3 -- `neg` is a built-in function that takes a I64 value and returns negative of it.
```

As I wrote before, there is no type of "two-variable functions" or "three-variable functions" in Fix. Instead, treat the value of type `a -> b -> c` (which is equal to `a -> (b -> c)`) as a thing like "two-variable function that takes a value of `a` and a value of `b`".　

Let's consider a "two-variable function" `multiply : I64 -> I64 -> I64` that multiplies two integers. Then `multiply(3) : I64 -> I64` is a function that multiplies 3 to the given integer. So `multiply(3)(5)` results in 15. Now, the last expression can be written as `multiply(3, 5)`, because we have a syntax sugar that `f(x, y)` is equivalent to `f(x)(y)`. 

In the program of Fibonacci sequence, the expression `Array::fill(n, 0)` is an example of calling two-variable function `Array::fill` on two values `n` and `0`.

As a special syntax, writing `f()` implies `f(())`, i.e., application of function `f` to the unit value `()`.

## Function definition

You can make a function value (which is similar to things called "lambda" or "closure" in other languages) by `|{arg}| {body}`. To define a two-variable function, you can simply write `|{arg0}, {arg1}| {body}` which is a syntax sugar of `|{arg0}| |{arg1}| {body}`.

Functions in fix can "capture" a value defined outside the function definition. As an example, consider the following program.

```
fifteen : I64;
fifteen = (
    let x = 3;
    let add_x = |n| n + x;
    add_x(4) + add_x(5) // (4 + 3) + (5 + 3) = 15
);
```

In the expression `|n| n + x`, `n` is the argument of the function and `x` refers to the name defined in the previous line. The function `add_x` memorises the value `3` and uses it when called.

Since all values (including functions) in Fix are immutable, the behavior of the function `add_x` will never change after you have defined it. For example, 

```
fifteen : I64;
fifteen = (
    let x = 3;
    let add_x = |n| n + x;
    let x = 0;
    add_x(4) + add_x(5) // (4 + 3) + (5 + 3) = 15
);
```

still evaluates to 15, because `add_x` is not affected by the change of the value that the name `x` refers to.

If the `{body}` part of your function ranges multiple lines, it is preferred to indent `{body}` with parenthes. For example, the program

```
calc_fib = |n| (
    let arr = Array::fill(n, 0);
    let arr = arr.set!(0, 1);
    let arr = arr.set!(1, 1);
    let arr = loop((2, arr), |(idx, arr)|
        if idx == arr.get_length {
            break $ arr
        } else {
            let x = arr.get(idx-1);
            let y = arr.get(idx-2);
            let arr = arr.set!(idx, x+y);
            continue $ (idx+1, arr)
        }
    );
    arr
);
```

is more readable than the following: 

```
calc_fib = |n| 
let arr = Array::fill(n, 0);
let arr = arr.set!(0, 1);
let arr = arr.set!(1, 1);
let arr = loop((2, arr), |(idx, arr)|
    if idx == arr.get_length {
        break $ arr
    } else {
        let x = arr.get(idx-1);
        let y = arr.get(idx-2);
        let arr = arr.set!(idx, x+y);
        continue $ (idx+1, arr)
    }
);
arr;
```

## Operator `.` and `$`

The operator `.` is another way of applying function to a value. It is defined as `x.f == f(x)`.

The precedence of the operator `.` is lower than function application by parenthes. So, if a function `method` has a type `Param -> Obj -> Result`, then `obj.method(arg)` is interpreted as `obj.(method(arg)) == method(arg)(obj) == method(arg, obj)`, not as `(obj.method)(arg)`.

In the program of Fibonacci sequence, the followings are examples of use of operator `.`:

- `arr.get_length`: `get_length` is a function of type `Array a -> I64`, which returns the length of an array. Note that you should not write `arr.get_length()` as if you call a method of a class on an instance in other languages. Remembering syntax sugars `f() == f(())` and `x.f == f(x)`, you can desugar the expression `arr.get_length()` to `get_length((), arr)`, which raises an error because `get_length` takes only one argument.
- `arr.set!(0, 1)`: `set!` is a function of type `I64 -> a -> Array a -> Array a`, which updates an element of an array to the specified value. 
- `arr.get(idx-1)`: `get` is a function of type `I64 -> Array a -> a`, which returns the element at the specified index.

We sometimes call a function of type `Param0 -> ... -> ParamN -> Obj -> Result` as a "method" on the type `Obj` that has N+1 parameters and returns a value of type `Result`. A method can be called by `obj.method(arg0, ..., argN)` as if writing OOP languages.

Another way of function application is operator `$`: `f $ x = f(x)`. This operator is right associative: `f $ g $ x = f(g(x))`. This operator is useful for reducing parenthes. In the program of Fibonacci sequence, the followings are examples of use of operator `$`:

- `continue $ (idx+1, arr)`: the application of the `continue` function to the tuple value `(idx+1, arr)`. In Fix, `continue` and `break` are usual functions, not syntaxes. So you can write this expression as `continue((idx+1, arr))` or `(idx+1, arr).continue`, but I prefer to write `continue $ (idx+1, arr)`, because it looks special. More explanation of `continue` and `break` functions will be given later. 
- `println $ Iterator::from_array(fib).map(to_string).join(", ")`: the application of the `println` function to the string expressed by `Iterator::from_array(fib).map(to_string).join(", ")`. The `println` function has type `String -> IO ()`, so applying to `println` to a string produces a value of `IO ()`, which is equal to the type of `main` function. This expression can also be written as `println(Iterator::from_array(fib).map(to_string).join(", "))`, but using operator `$` you can reduce parenthes around the long string expression.

The precedence between three ways of function application is `f(x)` > `x.f` > `f $ x`. By this, it is illegal to write `obj.method $ arg`. It is equivalent to `method(obj) $ arg" == method(obj, arg)`, which is trying to call `method` on two arguments in the wrong ordering. It is ok to write `method(arg) $ obj`, which can be read as "apply `method` to `arg` to obtain a function of type `Obj -> Result`, and apply it to `obj`" to get a result.

## Patterns

Both of let-expression and function expression introduces local names. If the type of the local name is tuple (or, more generally, structs), you can use patterns to destructure the passed value.

For example, let's define a function that takes a value of tuple type `(I64, Bool)`, and returns a value of `(Bool, I64)` by swapping two components. Using built-in functions `@0 : (a, b) -> a` and `@1 : (a, b) -> b` to extract the component from a tuple, you can write:

```
swap : (I64, Bool) -> (Bool, I64);
swap = |tuple| (
    let fst = tuple.@0;
    let snd = tuple.@1;
    (snd, fst)
);
```

Using pattern, this program can be written as:

```
swap : (I64, Bool) -> (Bool, I64);
swap = |tuple| (
    let (fst, snd) = tuple;
    (snd, fst)
);
```

or more shortly, 

```
swap : (I64, Bool) -> (Bool, I64);
swap = |(fst, snd)| (snd, fst);
```

Don't confuse `|(x, y)| ...` with `|x, y| ...`. The former defines a function that receives a tuple, where the latter defines a two-variable function.

## `loop`, `continue` and `break` function

The `loop` built-in function has type `s -> (s -> LoopResult s b) -> b`. The value of `LoopResult` type can be constructed from `continue` or `break` function.

- `continue : s -> LoopResult s b`
- `break : b -> LoopResult s b`

The `loop` function takes two arguments: the initial state of the loop `s0` and the loop body function `body`. It first calls `body` on `s0`. If `body` returns a value `break(r)`, then the `loop` function ends and returns `r` as the result. If `body` returns `continue(s)`, then the `loop` function calls again `body` on `s`.

In the program of Fibonacci sequence, the `loop` function is used in the following expression:

```
loop((2, arr), |(idx, arr)|
    if idx == arr.get_length {
        break $ arr
    } else {
        let x = arr.get(idx-1);
        let y = arr.get(idx-2);
        let arr = arr.set!(idx, x+y);
        continue $ (idx+1, arr)
    }
);
```

The initial value of this loop is `(2, arr)`. The loop body takes a tuple `(idx, arr)`, that is, the index of an array to be updated next, and an array to store the Fibonacci sequence whose values are already right at indices 0, ..., idx-1. If `idx` is less than `arr.get_length`, it calculates the value of Fibonacci sequence at `idx`, stores it to `arr`, and returns `continue $ (idx+1, arr)` to proceed to the next step. If `idx` has reached to `arr.get_length`, it returns `break $ arr` to end the loop. The return value of the `loop` function is an array.

## Unions

Then what is the type `LoopResult s b`? It is defined as an union with two type parameters `s` and `b`. It can be defined as follows:

```
type LoopResult s b = union { continue : s, break : b };
```

The above definition indicates that a `LoopResult s b` value contains either of a value of type `s` or a value of type `b`. If you write the set of values of a type as `|type|`, then `|LoopResult s b| = |s| ⨆ |b|`, where the symbol `⨆` is represents the disjoint union of sets.

For each union type, some basic methods are automatically defined. For example, for `LoopResult` as above, the following functions are defined in the namespace `LoopResult`.

- `continue : s -> LoopResult s b`: converts an value of type `s` into a `LoopResult` value.
- `break : b -> LoopResult s b`: converts an value of type `b` into a `LoopResult` value.
- `is_continue : LoopResult s b -> Bool`: checks if the `LoopResult` value was created by `continue`.
- `is_break : LoopResult s b -> Bool`: checks if the `LoopResult` value was created by `break`.
- `as_continue : LoopResult s b -> s`: extracts a value of type `s` from a `LoopResult` value if it is created by `continue`. If not, this function panics (i.e., prints an error message and stops the execution of the program).
- `as_break : LoopResult s b -> s`: extracts a value of type `b` from a `LoopResult` value if it is created by `break`. If not, this function panics (i.e., prints an error message and stops the execution of the program).

Another example of union is `Option` which is used to represent a value "which may not contain a value". It can be defined as follows: 

```
type Option a = union { none : (), some : s };
```

Note that, if you want to create a none value of `Option`, you need to write `none()`, because `none` is a function of type `() -> Option a`. (Remember that the syntax sugar `f() == f(())`.)

## Structs

Although it does not appear in the example Fibonacci program, here I explain how to define your own struct.

For example, you can define a struct called `Product` with two fields `price`  of type `I64` and `sold` of type `Bool` as follows.

```
type Product = struct { price: I64, sold: Bool };
```

You can construct a struct value by the syntax `{struct_name} { ({field_name}: {field_value}) } `:

```
let product = Product { price: 100, sold: false };
```

As in the case of unions, there are methods that are automatically defined for structs. For `Price` as above, the following methods are defined in the namespace `Price`.

- `@price : Product -> I64` and `@sold : Product -> Bool`
    - Extracts the value of a field from a `Product` value.
- `=price : I64 -> Product -> Product` and `=sold : Bool -> Product -> Product`
    - Modify a `Product` value by setting a field.
- `mod_price : (I64 -> I64) -> Product -> Product` and `mod_sold : (Bool -> Bool) -> Product -> Product`
    - Modify a `Product` value by a function acting on a field.

I already explained that we can use patterns to destructure tuples. You can also use patterns to destructure a struct value. For example, field accessor function `@price : Product -> I64` can be re-defined as follows: 

```
get_price : Product -> I64;
get_price = |product| (
    let Product { price: price, sold: sold } = product;
    price
);
```

or 

```
get_price : Product -> I64;
get_price = |Product { price: price, sold: sold }| price;
```

## Iterators

Now I explain about the expression `Iterator::from_array(fib).map(to_string).join(", ")`, where `fib : Array I64` is the array of Fibonacci sequence. This expression 
- converts a Fibonacci array into an iterator of integers, 
- apply `to_string : I64 -> String` to each element to obtain the iterator of strings, and
- concatenates these strings separated by `", "`,
- results in a string "1, 1, 2, 3, 5, 8, 13, 21, 34, 55, 89, 144, 233, 377, 610, 987, 1597, 2584, 4181, 6765, 10946, 17711, 28657, 46368, 75025, 121393, 196418, 317811, 514229, 832040".

Like array, iterator (a.k.a. "lazy list") is a way to represent sequences. Whereas an array stores the values of all elements in memory at the same time, an iterator only has a function to compute the next element and the next iterator. In fact, iterator in Fix is defined as follows:

```
type Iterator a = unbox struct { next: () -> Option (a, Iterator a) };
```

(You don't need to understand `unbox` specifier at now.)

The above definition indicates that the `Iterator` is a struct with only one field `next` of type `() -> Option (a, Iterator a)`.

The fundamental API (method) of `Iterator` is `advance` function, which just extract the `next` field from an iterator and calls it on `()`:
```
// Get next value and next iterator.
advance : Iterator a -> Option (a, Iterator a);
advance = |iter| (iter.@next)();
```

You can define an iterator that produces infinite sequence of zeros (0, 0, 0, ...) as follows: 
```
zeros : Iterator I64;
zeros = Iterator { next: |_| some $ (0, zeros) };
```

That is, if `advance` is called on `zeros`, it always returns `some` value (because it is an infinite sequence). If the programmer unwraps the `some` value, he obtains `0` as the value and `zeros` again as the next iterator.

```
let iter = zeros;
let (x, iter) = iter.advance.as_some; // x == 0
let (y, iter) = iter.advance.as_some; // y == 0
let (z, iter) = iter.advance.as_some; // z == 0
...
```

Since an iterator only has a function as a data, it consumes only a small memory. If we want to apply a function `f : a -> b` to each element of an array `arr : Array a` producing a new array of type `Array b`, we need to allocate an memory for the resulting array, which may be large. On the other hand, applying `f` to an iterator of `Iterator a` to produce an iterator of type `Iterator b` is faster and only needs small memory allocation, because any element of an iterator is not calculated until `advance` will be called. This operation is provided as `map` method of `Iterator`:

- `map : (a -> b) -> Iterator a -> Iterator b`

This can be defined as follows:

```
map : (a -> b) -> Iterator a -> Iterator b;
map = |f, iter| (
    let next = |_| (
        let adv = iter.advance;
        if adv.is_none { none() };
        let (val, iter_next) = adv.as_some;
        some $ (f(val), iter_next.map(f))
    );
    Iterator { next: next }
);
```

Going back to the Fibonacci program, there are more two functions related to `Iterator` used:

- `from_array : Array a -> Iterator a`: converts an array into an iterator.
- `join : String -> Iterator String -> String`: concatenates strings in an iterator separated by a specified string. NOTE: this is defined in `Std::String` namespace, not in `Std::Iterator`.

For example, `Iterator::from_array(["Hello", "World!"]).join(" ") == "Hello World!"`.

In the last, `to_string : I64 -> String` is a function that converts an integer to a decimal string.

## Mutation in Fix

In the last of this tutorial, I explain the meaning of the exclamation mark of `set!` function.

There is also a function without exclamation mark: `set : I64 -> a -> Array a -> Array a`. Semantically, both of `Array::set` and `Array::set!` return a new array with one element updated from the original array. 

Remember that an expression in Fix is only a sentence that describes a value. It is essentially the same as a mathematical expression such as "1 + cos(pi/5)^2". There is no concept of "changing the value of a variable" which is ubiquitous in usual languages. For example, consider

```
main = (
    let arr0 = Array::fill(100, 1);
    let arr1 = arr0.set(0, 2);
    println("arr0.get(0): " + arr0.get(0).to_string + ".")
);
```

The above prints `arr0.get(0): 1.`, not `2`. This is because `arr0.set(0, 2)` is merely an expression that says "an array which is almost identical to `arr0` but with the 0th element replaced by `2`", and it is NOT a command "update the 0th element of `arr0` to `2`". To realize this behavior, `set` function in the above program has to clone `arr0` before updating the 0th element of an array.

More generally, all values of Fix are immutable. Immutability is good for reducing bugs caused by fails on state management, but it can be an obstacle for implementing an algorithm with its optimum time (or space) complexity. Consider the implementation of `calc_fib` function of the example program using `set` instead of `set!`:

```
calc_fib : I64 -> Array I64;
calc_fib = |n| (
    let arr = Array::fill(n, 0);
    let arr = arr.set(0, 1);
    let arr = arr.set(1, 1);
    let arr = loop((2, arr), |(idx, arr)|
        if idx == arr.get_length {
            break $ arr
        } else {
            let x = arr.get(idx-1);
            let y = arr.get(idx-2);
            let arr = arr.set(idx, x+y);
            continue $ (idx+1, arr)
        }
    );
    arr
);
```

The optimum time complexity of calculating Fibonacci sequence of length N is O(N). But if Fix had cloned the array at `let arr = arr.set(idx, x+y);` in the loop, it takes O(N) time for each loop step and the total time complexity becomes O(N^2).

In fact, `set` in the above program doesn't clone the array and `calc_fib` works in O(N) time, as expected. This is because if the given array will no longer be used, `set` omits cloning and just updates the given array. Let's consider a simpler program: 

```
main = (
    let arr0 = Array::fill(100, 1);
    let arr1 = arr0.set(0, 2);
    println("arr1.get(0): " + arr1.get(0).to_string + ".")
);
```

(Note that `println` prints the 0th element of `arr1`, not of `arr0`.) In this program, the call of `set` is the last usage of `arr0`. In such a case, `set` can update the 0th element of the given array without violating immutability, because the mutation cannot be observed. 

Go back to the `calc_fib` function. At the line `let arr = arr.set(idx, x+y);`, the name `arr` is redefined and set as pointing to the new array returned by `set` function. This ensures that the old array given to `set` function will be never referenced after this line. So it is evident that `set` function doesn't need to clone the given array, and in fact it doesn't.

As a summary, since values in Fix are immutable, the `set : I64 -> a -> Array a -> Array a` function basically returns a new array with one element replaced, but it omits cloning an array if the array will not be used later.

The `set!` function is almost same as the `set` function, but it panics (i.e., stop the execution of the program) if the given array will be used later. In other words, there is assurance that `set!` doesn't clone the array. This is useful to assure that a program is running at a expected time complexity. We put the exclamation mark for a function that requires the assurance that the given value will not be used later.

# Other topics on syntax

## Module and imports 

In Fix, values, functions, types and traits defined in a source file is collected to a module. Each source file has to declare the name of the module it defines by `module {module_name};`. The first letter of the module name must be capitalized.

As in other languages, a single program can be constructed from multiple source files. As an example, consider a program consists of two source files:

`lib.fix`:
```
module Lib;

module_name : String;
module_name = "Lib";
```

`main.fix`:
```
module Main;

import lib.fix;

module_name : String;
module_name = "Main";

main : IO ();
main = (
    println $ "This program consists of two modules, `" + Lib::module_name + "` and `" + Main::module_name + "`."
);
```

If you put these two files in a same directory and execute `fix run main.fix`, it prints: 

```
This program consists of two modules, `Lib` and `Main`.
```

Note that here two strings named `module_name` are defined and you can use these strings separately by writing `{module_name}::module_name`. Like this, module name is used as the top-level namespace of values, types and traits defined in a source file.

You can import modules defined in other source files by writing `import {path_to_source_file};`. If `{path_to_source_file}` starts by `./` or `../`, then it is treated as a relative path to the source file in which the import statement is written. In other cases, `{path_to_source_file}` is treated as a relative path to the root source file, that is, the file passed to the `fix run` or `fix build` command.

## Recursion

You can make recursive global function as in usual programming languages.

```
module Main;

fib : I64 -> I64;
fib = |n| (
    if n == 0 {
        0
    } else if n == 1 {
        1
    } else {
        fib(n-1) + fib(n-2)
    }
);

main : IO ();
main = print $ fib(30).to_string; // 832040
```

On the other hand, Fix's `let`-binding doesn't allow to make recursive definition. To define a recursive function locally, use `fix` built-in function.

## Overloading

(TBA)

## Trait

(TBA)

## Monad

### What is monad?

The trait `Monad` is defined as follows:

```
trait [m : *->*] m : Monad {
    bind : (a -> m b) -> m a -> m b;
    pure : a -> m a;
}
```

In the following sections, we introduce 3 typical kinds of monads used practically.

#### State-like monads

This kind of monad represents an "action" (a computation in an environment). In Fix's standard library, `IO` is a state-like monad where `IO a` represents an I/O action that returns a value of type `a`. As another example, the following definition

```
type State s a = unbox struct { run : s -> (s, a) }
```

produces a monad `State s`. This monad represents a computation which reads and updates the "state", which ia a value of `s`.

For state-like monads, `bind` provides a way to combine two actions. An action `x.bind(f)` represents the following action:
- First, perform the action `x`. Let `r` denote the result of the action `x`.
- Then, perform the action `f(r)`.
 
An action `pure(v)` represents a computation that returns `v` with no interaction with the environment.

For example, `print(str) : IO ()` is an I/O action that prints `str` to the standard output. Assume that `read : IO String` is an I/O action that reads a content of standard input as a string. Then, the I/O action `echo` that reads standard input and just prints it can be written as:

```
echo : IO ();
echo = read.bind(|s| print(s));
```

NOTE: Actually there is no `read : IO String` defined in Fix's standard library. It can be defined as `read_content(stdin).map(as_ok)`.

#### Result-like monads

This kind of monad represents a value that may fail to be calculated. In Fix's standard library, `Result e` is a monad with an error `e`:

```
type Result e o = unbox union { ok : o, err: e };
```

`Result e o` contains a successful value of type `o`, or an error value of type `e`. Another example is the `Option` monad:

```
type Option a = union { none: (), some: a };
```

For result-like monads, `bind` provides a way to do short-circuit evaluation. `x.bind(f)` should immediately return an error (or "none") value if `x` is an error. Only when `x` is an ok (or "some") value `v`, the function `f` is called and `x.bind(f)` should evaluates to `f(v)`. `pure(v)` represents an ok value `v`.

As an example, consider a function `add_opt : Option I64 -> Option I64 -> Option I64` which adds two integers only when both are "some" values. Naively, it can defined as follows:

```
add_opt : Option I64 -> Option I64 -> Option I64;
add_opt = |x, y| (
    if x.is_none { Option::none() };
    let x = x.as_some;
    if y.is_none { Option::none() };
    let y = y.as_some;
    Option::some(x+y)
);
```

Using `bind`, the above program can be rewritten as:

```
add_opt : Option I64 -> Option I64 -> Option I64;
add_opt = |x, y| x.bind(|x| y.bind(|y| Option::some(x+y)));
```

#### List-like monads

In Fix's standard library, `Iterator` is an example of list-like monad. For list-like moads, `[x, y, z, ...].bind(f)` represents `f(x) + f(y) + f(z) + ...`, where `+` concatenates two list-like values. `pure(x)` represents an singleton value `[x]`. 

NOTE: In fact `[a,b,c,...]` is an array literal, but here we are writing it as literal for list-like values.

For example, consider a function `product : Iterator a -> Iterator b -> Iterator (a, b)` that calculates a cartesian product. It can be implemented as:

```
product : Iterator a -> Iterator b -> Iterator (a, b);
product = |xs, ys| xs.bind(|x| ys.bind(|y| pure $ (x, y)));
```

because, if `xs == [x0, x1, ...]` and `ys == [y0, y1, ...]`, then 

```
xs.bind(|x| ys.bind(|y| pure $ (x, y)))
== ys.bind(|y| pure $ (x0, y)) + ys.bind(|y| pure $ (x1, y)) + ...
== (pure $ (x0, y0)) + (pure $ (x0, y1)) + ... + (pure $ (x1, y0)) + (pure $ (x1, y1)) + ... + ...
== [(x0, y0)] + [(x0, y1)] + ... + [(x1, y0)] + [(x1, y1)] + ... + ...
== [(x0, y0), (x0, y1), ..., (x1, y0), (x1, y1), ..., ...]
```

### `do` block and monadic bind operator `*`

A prefix unary operator `*` provides a way to use `bind` in more concise way. A code `B(*x)` is expanded to `x.bind(|v| B(v))`. Here, `B(*x)` is the minimal `do` block that encloses the expression `*x`. Here, `do` blocks are defined as follows:

- You can make `do` block explicitly by `do { ... }`.
- Lambda-expression `|arg| ...` defines a `do` block `...` implicitly.
- Let-definition `let name = val in ...` defines a `do` block `...` implicitly.
- If-expression `if cond { ... } else { ... }` defines two blocks  `...` implicitly.
- Global definition `name = ...` defines a `do` block `...` implicitly.

Examples in previous sections can be written using `*` as follows:

```
echo : IO ();
echo = print(*read);
```

```
add_opt : Option I64 -> Option I64 -> Option I64;
add_opt = |x, y| pure $ *x + *y;
```

```
product : Iterator a -> Iterator b -> Iterator (a, b);
product = |xs, ys| pure $ (*xs, *ys);
```

The following is an example where you need to make `do` block explicitly.

```
add_opt_unwrap : Option I64 -> Option I64 -> I64;
add_opt_unwrap = |x, y| do { pure $ *x + *y }.as_some;
```

In the above, the definition of `add_opt_unwrap` will be appropriately expanded to 

```
add_opt_unwrap = x.bind(|x| y.bind(|y| pure $ x + y)).as_some;
```

On the other hand, if you write 

```
add_opt_unwrap = |x, y| (pure $ *x + *y).as_some;
```

it will be expanded to 

```
add_opt_unwrap = |x, y| x.bind(|x| y.bind(|y| (pure $ x + y).as_some));
```

which won't be compiled, because the inner `bind` requires a function that returns `Option I64` but the function `|y| (pure $ x + y).as_some` has type `I64 -> I64`.

## Type annotation

(TBA)

## Boxed and unboxed types

Types in Fix are divided into boxed types and unboxed types. Boxed types and unboxed types are similar to things called as "reference types" and "value types" in other languages, respectively.

* Value of boxed types are allocated in heap memory. Local names and struct / union fields whose types are boxed are compiled as pointers to the values. 
* Values of unboxed types are directly embedded into the stack memory, structs and unions. 

In general, types that contain a lot of data (such as `Array`) are suited to be boxed because boxed types have lower copying costs. On the other hand, types containing small data (such as `I64`) can be unboxed to reduce the cost of increasing or decreasing the reference counter.

### Functions

Functions are unboxed, but captured values are stored to an unnamed boxed struct.

### Tuples

Tuple types are unboxed, because tuple is intended to have only a few fields. If you want to use many fields, you should define a new struct.
Tuples are special forms of structs whose field names are `0`, `1`, `2`, etc. 

### Unit

The unit type `()` is unboxed.

### Array

`Std::Array` is a boxed type.

### Structs

Structs are boxed by default because they are assumed to have many fields. To define unboxed struct type, write `unbox` specifier before `struct`.

Example:
```
type Product = unbox struct { price: I64, sold: Bool };
```

### Unions

Unions are unboxed by default because they only contains a single value at a time. To define boxed union type, write `box` specifier before `struct`.

```
type Weight = box union { pound: I64, kilograms: I64 };
```

## Calling C functions

To call C functions, use the following expression:

```
CALL_C[{c_function_signature}, {arg_0}, {arg_1}, ...]
```

Example: 

```
main : IO ();
main = (
    let _ = "Hello C function!\n".call_with_c_str(|ptr|
        CALL_C[I32 printf(Ptr, ...), ptr]
    );
    pure()
);
```

In `{c_function_signature}`, you need to specify type of return value and arguments. 

- Use `Ptr` for pointers.
- Use `U8`, `I32`, `U32`, `I64`, `U64` for integral types.
- Use `...` for `va_arg`.
- If return type is `void`, put `()` before the function name.

Note that calling C function may break abstraction of Fix such as immutability or memory safety. Use this feature carefully!

# Built-in / library features

## Types

### Structs

If you define a struct named `{struct}` with a field `{field_name}` of type `{field_type}`, the following methods are defined in the namespace named `{struct}`.

NOTE: In a future, we will add lens functions such as `act_{field_name} : [f: Functor] ({field_type} -> f {field_type}) -> {struct} -> f {struct} `, which are generalization of `mod` functions.

#### `@{field_name} : {struct} -> {field_type}`

Extract the value of a field from a struct value.

#### `={field_name} : {field_type} -> {struct} -> {struct}`

Modify a struct value by setting a field.
This function clones the struct value if it is shared between multiple references.

#### `={field_name}! : {field_type} -> {struct} -> {struct}`

Modify a struct value by setting a field.
This function always updates the struct value. If the struct value is shared between multiple references, this function panics.

#### `mod_{field_name} : ({field_type} -> {field_type}) -> {struct} -> {struct}`

Modify a struct value by a function acting on a field.
This function clones the struct value if it is shared between multiple references.
It is assured that if you call `obj.mod_field(f)` when the reference counter of the field value in `obj` is one, then `f` receives the field value uniquely.

#### `mod_{field_name}! : ({field_type} -> {field_type}) -> {struct} -> {struct}`

This function is almost same as `mod_{field_name}` except that this function asserts uniqueness of given struct value.
This function always updates the struct value. If the struct value is shared between multiple references, this function panics.

### Unions

If you define a union named `{union}` with a variant `{variant_name}` of type `{variant_type}`, the following methods are defined in the namespace named `{union}`.

#### `{variant_name} : {variant_type} -> {union}`

Constructs a union value from a variant value.

#### `is_{variant_name} : {union} -> Bool`

Check if a union value is created as the specified variant.

#### `as_{variant_name} : {union} -> {variant_type}`

Converts a union value into a variant value if it is created as the variant. If not so, this function panics.

#### `mod_{variant_name} : ({variant_type} -> {variant_type}) -> {union} -> {union}`

Modify a union value by a function acting on a variant. It is assured that if you call `obj.mod_variant(f)` when the reference counter of the variant value in `obj` is one, then `f` receives the variant value uniquely.

### Std::Array

`Std::Array` is the type of variable-length arrays.

Literals: 
- `[{elem_0}, {elem_1}, ...]`
    - Example: `[1, 2, 3]` for integer array of length 3.

Methods:

#### `__unsafe_set_length : I64 -> Array a -> Array a`
Updates the length of an array, without uniqueness checking or validation of the given length value.

#### `__unsafe_get : I64 -> Array a -> a`
Gets a value from an array, without bounds checking and retaining the returned value.

#### `__unsafe_set : I64 -> a -> Array a -> Array a`
Sets a value into an array, without uniqueness checking, bounds checking and releasing the old value.

#### `_get_ptr : Array a -> Ptr`
Get the pointer to the memory region where elements are stored.
Note that in case the array is not used after call of this function, the returned pointer will be already released.

#### `_sort_range_using_buffer : Array a -> I64 -> I64 -> ((a, a) -> Bool) -> Array a -> (Array a, Array a)`
Sort elements in a range of an array by "less than" comparator.
This function receives a working buffer as the first argument to reduce memory allocation, and returns it as second element.

#### `append : Array a -> Array a -> Array a`
Append an array to an array.
Note: Since `a1.append(a2)` puts `a2` after `a1`, `append(lhs, rhs)` puts `lhs` after `rhs`. 

#### `call_with_ptr : (Ptr -> b) -> Array a -> b`
Call a function with a pointer to the memory region where elements are stored.

#### `empty : I64 -> Array a`
Creates an empty array with specified capacity.

#### `fill : I64 -> a -> Array a`
Creates an array filled with the initial value.
The capacity is set to the same value as the length.
Example: `fill(n, x) == [x, x, x, ..., x]` (of length `n`).

#### `force_unique : Array a -> Array a`
Force the uniqueness of an array.
If the given array is shared, this function returns the cloned array.

#### `force_unique! : Array a -> Array a`
Force the uniqueness of an array.
If the given array is shared, this function panics.

#### `from_map : I64 -> (I64 -> a) -> Array a`
Creates an array by a mapping function.
Example: `from_map(n, f) = [f(0), f(1), f(2), ..., f(n-1)]`.

#### `get : I64 -> Array a -> a`
Returns an element of an array at an index.

#### `get_first : Array a -> Option a`
Get the first element of an array. Returns none if the array is empty.

#### `get_last : Array a -> Option a`
Get the last element of an array. Returns none if the array is empty.

#### `get_length : Array a -> I64`
Returns the length of an array.

#### `get_capacity : Array a -> I64`
Returns the capacity of an array.

#### `is_empty : Array a -> Bool`
Returns if the array is empty or not.

#### `mod : I64 -> (a -> a) -> Array a -> Array a`
Modifies a value of an element at the specified index of an array by a function.
This function clones the array if it is shared between multiple references.

#### `mod! : I64 -> (a -> a) -> Array a -> Array a`
This function clones the array if it is shared between multiple references.
This function always update the array. If the array is shared between multiple references, this function panics.  

#### `pop_back : Array a -> Array a`
Pop an element at the back of an array.
If the array is empty, this function does nothing.

#### `push_back : a -> Array a -> Array a`
Push an element to the back of an array.

#### `range : I64 -> I64 -> Iterator I64`
Create a range iterator, i.e. an iterator of the form `[a, a+1, a+2, ..., b-1]`.

#### `reduce_length : I64 -> Array a -> Array a`
Reduce the length of an array.

#### `reserve : I64 -> Array a -> Array a`
Reserves the memory region for an array.

#### `set : I64 -> a -> Array a -> Array a`
Updates a value of an element at an index of an array.
This function clones the given array if it is shared between multiple references.

#### `set! : I64 -> a -> Array a -> Array a`
Updates a value of an element at an index of an array.
This function always update the given array. If the given array is shared between multiple references, this function panics.

#### `sort_by : ((a, a) -> Bool) -> Array a -> Array a`
Sort elements in an array by "less than" comparator.

You can create array by the array literal syntax `[a0, a1, ..., an]`.

NOTE: In a future, we will add lens functions such as `act : [f: Functor] I64 -> (a -> f a) -> Array a -> f (Array a)`, which are generalization of `mod` functions.

Implementing Traits:

- `[a : Eq] Array a : Eq`

### Std::Bool

`Std::Bool` is the type of boolean values, represented by 8-bit integer `1` (`true`) and `0` (`false`). 

Boolean literals are `true` and `false`.

#### `impl Bool : Eq`
#### `impl Bool : ToString`

### Std::F32

`F32` is the type of 32-bit floating numbers.

For `F32` literals, you need to add a suffix "_F32" to explicitly specify the type. Example: `3.1416_F32`.

#### `abs : F32 -> F32`
#### `impl F32 : Add`
#### `impl F32 : Div`
#### `impl F32 : Eq`
#### `impl F32 : LessThan`
#### `impl F32 : LessThanOrEq`
#### `impl F32 : Mul`
#### `impl F32 : Sub`
#### `impl F32 : ToF32`
#### `impl F32 : ToF64`
#### `impl F32 : ToString`

### Std::F64

`F64` is the type of 64-bit floating numbers.

For `F64` literals, you can write or omit explicit type specifier suffix "_F64". Example `3.1416_F64 == 3.1416`.

#### `abs : F64 -> F64`
#### `impl F64 : Add`
#### `impl F64 : Div`
#### `impl F64 : Eq`
#### `impl F64 : LessThan`
#### `impl F64 : LessThanOrEq`
#### `impl F64 : Mul`
#### `impl F64 : Sub`
#### `impl F64 : ToF32`
#### `impl F64 : ToF64`
#### `impl F64 : ToString`

### Std::IO

`IO a` is the type whose value represents an I/O action which returns a value of type `a`.

#### `__unsafe_perform : IO a -> a`

Perform the I/O action. This may violate purity of Fix.

#### `close_file : IOHandle -> IO ()`

Close a file.

#### `open_file : Path -> String -> IOResult IOError IOHandle`

Open a file. The second argument is a mode string for `fopen` C function. 

#### `print : String -> IO ()`

Print a string to the standard output.

#### `println : String -> IO ()`

Print a string followed by a newline to the standard output.

#### `read_content : IOHandle -> IOResult IOError String`

Read all characters from a IOHandle.

#### `read_file : Path -> IOResult IOError String`

Raad all characters from a file.

#### `read_line : IOHandle -> IOResult IOError String`

Read characters from a IOHandle upto newline/carriage return or EOF. The returned string may include newline/carriage return at it's end.

Example: 
```
module Main;

main : IO ();
main = (
    let Result::ok(str) = *read_line(stdin);
    println(str)
);
```

#### `read_line_inner : Bool -> IOHandle -> IOResult IOError String`

Read characters from an IOHandle.
if the first argument `upto_newline` is true, this function reads a file upto newline/carriage return or EOF.

#### `with_file : Path -> String -> (IOHandle -> IOResult IOError a) -> IOResult IOError a`

Perform a function with a file handle. The second argument is a mode string for `fopen` C function. 
The file handle will be closed automatically.

#### `write_content : IOHandle -> String -> IOResult IOError ()`

Write a string into an IOHandle.

#### `write_file : Path -> String -> IOResult IOError ()`

Write a string into a file.

#### `impl IO : Functor`

#### `impl IO : Monad`

### Std::IO::IOError

A type for I/O error.

```
type IOError = unbox struct { msg : String };
```

#### `impl IOError : ToString`

Returns the value of `msg` field.

### Std::IO::IOHandle

A handle type for read / write operations on files/stdin/stdout/stderr.

#### `stderr : IOHandle`

The handle for standard error.

#### `stdin : IOHandle`

The handle for standard input.    

#### `stdout : IOHandle`

The handle for standard output.

### Std::IO::IOResult

The type of I/O actions which may fail.

```
type IOResult e a = unbox struct { _data : IO (Result e a) };
```

#### `from_result : Result e a -> IOResult e a`

Create a constant IOResult from a Result value.

#### `lift : IO a -> IOResult e a`

Lift an IO action to a successful IOResult.

#### `to_io : IOResult e a -> IO (Result e a)`

Convert an IOResult to an IO action.

#### `impl IOResult e : Functor`

#### `impl IOResult e : Monad`

### Std::I32

`Std::I32` is the type of 32-bit signed integers.

Literals:
- `{number}_I32`
    - Example: `42_I32`

Implementing traits:

- `Std::Add`
- `Std::Eq`
- `Std::LessThan`
- `Std::LessThanOrEq`
- `Std::Mul`
- `Std::Neg`
- `Std::Rem`
- `Std::Sub`
- `Std::ToString`
- `Std::ToU8`
- `Std::ToI32`
- `Std::ToU32`
- `Std::ToI64`
- `Std::ToU64`

#### _I32_to_string : I32 -> String

### Std::I64

`Std::I64` is the type of 64-bit signed integers.

Literals:
- `{number}`
    - Example: `42`
- `{number}_I64`
    - Example: `42_I64 == 42`

Implementing traits:

- `Std::Add`
- `Std::Eq`
- `Std::LessThan`
- `Std::LessThanOrEq`
- `Std::Mul`
- `Std::Neg`
- `Std::Rem`
- `Std::Sub`
- `Std::ToString`
- `Std::ToU8`
- `Std::ToI32`
- `Std::ToU32`
- `Std::ToI64`
- `Std::ToU64`

#### _I64_to_string : I64 -> String

### Std::Iterator

Iterators (a.k.a. lazy lists) are generators of sequenced values.

#### `advance : Iterator a -> Option (a, Iterator a)`
Get next value and next iterator.

#### `append : Iterator a -> Iterator a -> Iterator a`
Append an iterator to a iterator.
Note: Since `iter1.append(iter2)` puts `iter2` after `iter1`, `append(lhs, rhs)` puts `lhs` after `rhs`.    

#### `count_up : I64 -> Iterator I64`
Create an iterator that counts up from a number.
Example: `count_up(n) = [n, n+1, n+2, ...]` (continues infinitely).

#### `empty : Iterator a`
Create an empty iterator.

#### `get_length : Iterator a -> I64`
Counts the length of an iterator.

#### `intersperse : a -> Iterator a -> Iterator a`

Intersperse an elemnt between elements of an iterator.
Example: `Iterator::from_array([1,2,3]).intersperse(0) == Iterator::from_array([1,0,2,0,3])`

#### `is_empty : Iterator a -> Bool`

Check if the iterator is empty.

#### `filter : (a -> Bool) -> Iterator a -> Iterator a`
Filter elements by a condition function.

#### `fold : b -> (b -> a -> b) -> Iterator a -> b`
Folds iterator from left.
Example: `fold(init, op, [a0, a1, a2, ...]) = ...op(op(op(init, a0), a1), a2)...`.

#### `from_array : Array a -> Iterator a`
Create iterator from an array.

#### `from_map : (I64 -> a) -> Iterator a`
Create iterator from mapping function.
Example: `from_map(f) = [f(0), f(1), f(2), ...]`.

#### `push_front : a -> Iterator a -> Iterator a`
Append an element to an iterator.

#### `reverse : Iterator a -> Iterator a`
Reverse an iterator.

#### `take : I64 -> Iterator a -> Iterator a`
Take at most n elements from an iterator.

#### `zip : Iterator a -> Iterator b -> Iterator (a, b)`
Zip two iterators.

#### `impl Iterator a : Add`
Adds two iterators by `Iterator::append`.

#### `impl [a : Eq] Iterator a : Eq`

#### `impl Iterator : Functor`

#### `impl Iterator : Monad`

### Std::Option

`Option a` contains a value of type `a`, or contains nothing.

```
type Option a = union { none: (), some: a };
```

#### `impl [a : Eq] Option a : Eq`

#### `impl Option : Functor`

#### `impl Option : Monad`

### Std::Path

The type for file path.

Implementing traits:

- `Path : ToString`

#### `parse : String -> Option Path`

Parse a string.

### Std::Ptr

`Std::Ptr` is the type of pointers.

Literals:
- `nullptr`
    - The null pointer.

Implementing traits:

- `Ptr : Eq`

### Std::Result

A type of result value for a computation that may fail.

```
type Result o e = unbox union { ok : o, err: e };
```

#### `unwrap : [e : ToString] Result e o -> o`

Returns the containing value if the value is ok, or otherwise panics after printing error value.

#### `impl Result e : Monad`

### Std::String

The type of strings.

#### `_get_c_str : String -> Ptr`
Get the null-terminated C string.
Note that in case the string is not used after call of this function, the returned pointer will be already released.

#### `call_with_c_str : (Ptr -> a) -> String -> a`
Call a function with a valid null-terminated C string.

#### `concat : String -> String -> String`
Concatenate two strings.
Note: Since `s1.concat(s2)` puts `s2` after `s1`, `concat(lhs, rhs)` puts `lhs` after `rhs`.

#### `concat_iter : Iterator String -> String`
Concatenate an iterator of strings.

#### `get_first_byte : String -> Option Byte`
Get the first byte of a string. Returns none if the string is empty.

#### `get_last_byte : String -> Option Byte`
Get the last byte of a string. Returns none if the string is empty.

#### `get_length : String -> I64`
Returns the length of the string.

#### `is_empty : String -> Bool`
Returns if the string is empty or not.

#### `join : String -> Iterator String -> String`
Join strings by a separator.
Example: `Iterator::from_array(["a", "b", "c"]).join(", ") == "a, b, c"`

#### `pop_back_byte : String -> String`
Removes the last byte.
If the string is empty, this function does nothing.

#### `strip_last_bytes : (Byte -> Bool) -> String -> String`
Removes newlines and carriage returns at the end of the string.

#### `strip_last_newlines : String -> String`
Removes the last byte of a string while it satisifies the specified condition.

Implementing Traits:

- `String : Add`
    - Add two strings by `String.concat`.
- `String : Eq`
- `String : ToString`
    - Defined as an identity function.

### Std::U8

`Std::U8` is the type of 8-bit unsigned integers.

Literals:

- `{number}_U8`
    - Example: `-1_U8 == 255_U8`
- `'{character}'`
  - Example: `'A' == 65_U8`, `'\n' == 10_U8`, `'\x7f' == 127_U8`

Implementing traits:

- `Std::Add`
- `Std::Eq`
- `Std::LessThan`
- `Std::LessThanOrEq`
- `Std::Mul`
- `Std::Neg`
- `Std::Rem`
- `Std::Sub`
- `Std::ToString`
- `Std::ToU8`
- `Std::ToI32`
- `Std::ToU32`
- `Std::ToI64`
- `Std::ToU64`

#### _U8_to_string : U8 -> String

### Std::U32

`Std::U32` is the type of 32-bit unsigned integers.

Literals:

- `{number}_U32`
    - Example: `-1_U32 == 4294967295_U32`

Implementing traits:

- `Std::Add`
- `Std::Eq`
- `Std::LessThan`
- `Std::LessThanOrEq`
- `Std::Mul`
- `Std::Neg`
- `Std::Rem`
- `Std::Sub`
- `Std::ToString`
- `Std::ToU8`
- `Std::ToI32`
- `Std::ToU32`
- `Std::ToI64`
- `Std::ToU64`

#### _U32_to_string : U32 -> String

### Std::U64

`Std::U64` is the type of 64-bit unsigned integers.

Literals:

- `{number}_U64`
    - Example: `-1_U64 == 18446744073709551615_U64`

Implementing traits:

- `Std::Add`
- `Std::Eq`
- `Std::LessThan`
- `Std::LessThanOrEq`
- `Std::Mul`
- `Std::Neg`
- `Std::Rem`
- `Std::Sub`
- `Std::ToString`
- `Std::ToU8`
- `Std::ToI32`
- `Std::ToU32`
- `Std::ToI64`
- `Std::ToU64`

#### _U64_to_string : U64 -> String

## Functions

### Std::is_unique : a -> (Bool, a)

This function checks if a value is uniquely refernced by a name, and returns the pair of the result and the given value. If `a` is unboxed, the 0th component of the returned value will be `true`.

Example: 

```
main : IO ();
main = (
    // For unboxed value, it returns true even if the value is used later.
    let int_val = 42;
    let (unique, _) = int_val.is_unique;
    let use = int_val + 1;
    let _ = assert_eq("fail: int_val is shared", unique, true);

    // For boxed value, it returns true if the value isn't used later.
    let arr = Array::fill(10, 10);
    let (unique, arr) = arr.is_unique;
    let use = arr.get(0); // This `arr` is not the one passed to `is_unique`, but the one returned by `is_unique`.
    let _ = assert_eq("fail: arr is shared", unique, true);

    // Fox boxed value, it returns false if the value will be used later.
    let arr = Array::fill(10, 10);
    let (unique, _) = arr.is_unique;
    let use = arr.get(0);
    let _ = assert_eq("fail: arr is unique", unique, false);

    pure()
);
```

### Std::fix : ((a -> b) -> a -> b) -> a -> b

`fix` enables you to make a recursive function locally. The idiom is: `fix $ |loop, var| -> (expression calls loop)`.

```
module Main;

main : IO ();
main = (
    let fact = fix $ |loop, n| if n == 0 then 1 else n * loop (n-1);
    print $ fact(5).to_string // evaluates to 5 * 4 * 3 * 2 * 1 = 120
);
```

### Std::loop : s -> (s -> LoopResult s r) -> r

`loop` enables you to make a loop. `LoopResult` is a union type defined as follows: 

```
type LoopResult s r = union { s: continue, r: break };
```

`loop` takes two arguments: the initial state of the loop `s0` and the loop body function `body`. It first calls `body` on `s0`. If `body` returns `break r`, then the loop ends and returns `r` as the result. If `body` returns `continue s`, then the loop calls again `body` on `s`.

```
module Main;
    
main : IO ();
main = (
    let sum = (
        loop((0, 0), |(i, sum)|
            if i == 100 then 
                break $ sum 
            else
                continue $ (i+1, sum+i)
        )
    );
    print $ sum.to_string
); // evaluates to 0 + 1 + ... + 99 
```

### Std::Debug::debug_print : String -> ()

### Std::Debug::debug_println : String -> ()

### Std::Debug::abort : () -> a

### Std::Debug::assert : String -> Bool -> ()

### Std::Debug::assert_eq : [a: Eq] String -> a -> a -> ()

## Traits

### Std::Functor (* -> *)

#### `map : [f : Functor] (a -> b) -> f a -> f b`

### Std::Monad (* -> *)

#### (required) `bind : [m : Monad] (a -> m b) -> m a -> m b`

#### `flatten : [m : Monad] m (m a) -> a`

This is equivalent to `Monad::bind(|x|x)`.

#### (required) `pure : [m : Monad] a -> m a`

### Std::ToString

#### `to_string : [a: ToString] a -> String`

### Std::ToI32

#### `to_I32 : [a: ToI32] a -> I32`

### Std::ToI64

#### `to_I64 : [a: ToI64] a -> I64`

### Std::ToU8

#### `to_U8 : [a: ToU8] a -> U8`

### Std::ToU32

#### `to_U32 : [a: ToU32] a -> U32`

### Std::ToU64

#### `to_U64 : [a: ToU64] a -> U64`

## Operators

The following is the table of operators sorted by it's precedence (operator of higher precedence appears earlier).

| Operator / syntax | Type                     | Trait / method                      | Explanation                                                 |
| ----------------- | ------------------------ | ----------------------------------- | ----------------------------------------------------------- |
| f(x)              | syntax                   | -                                   | function application                                        |
| .                 | left associative binary  | -                                   | right-to-left function application: x.f = f(x)              |
| *                 | unary prefix             | Std::Monad / bind                   | monadic bind                                                |
| - (minus sign)    | unary prefix             | Std::Neg / neg                      | negative of number                                          |
| !                 | unary prefix             | Std::Not / not                      | logical NOT                                                 |
| *                 | left associative binary  | Std::Mul / mul                      | multiplication of numbers                                   |
| /                 | left associative binary  | Std::Div / div                      | division of numbers                                         |
| %                 | left associative binary  | Std::Rem / rem                      | reminder of division                                        |
| +                 | left associative binary  | Std::Add / add                      | addition of numbers                                         |
| - (minus sign)    | left associative binary  | Std::Sub / sub                      | subtraction of numbers                                      |
| ==                | left associative binary  | Std::Eq / eq                        | equality comparison                                         |
| !=                | left associative binary  | -                                   | `x != y` is interpreted as `!(x == y)`                      |
| <=                | left associative binary  | Std::LessThanOrEq / less_than_or_eq | less-than-or-equal-to comparison                            |
| >=                | left associative binary  | -                                   | `x >= y` is interpreted as `y <= x`                         |
| <                 | left associative binary  | Std::LessThan / less_than           | less-than comparison                                        |
| >                 | left associative binary  | -                                   | `x > y` is interpreted as `y < x`                           |
| &&                | right associative binary | -                                   | short-circuit logical AND.                                  |
| &#124;&#124;      | right associative binary | -                                   | short-circuit logical OR                                    |
| $                 | right associative binary | -                                   | right associative function application: f $ g $ x = f(g(x)) |