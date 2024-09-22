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
  - [Mutation in Fix and reference counter](#mutation-in-fix-and-reference-counter)
- [Other topics on syntax](#other-topics-on-syntax)
  - [Boolean values and literals](#boolean-values-and-literals)
  - [Numbers and literals](#numbers-and-literals)
  - [Strings and literals](#strings-and-literals)
  - [Arrays and literals](#arrays-and-literals)
  - [Unit and tuples](#unit-and-tuples)
  - [Structs](#structs-1)
    - [`@f : S -> F`](#f--s---f)
    - [`set_f : F -> S -> S`](#set_f--f---s---s)
    - [`mod_f : (F -> F) -> S -> S`](#mod_f--f---f---s---s)
    - [`act_f : [f : Functor] (F -> f F) -> S -> f S`](#act_f--f--functor-f---f-f---s---f-s)
  - [Unions](#unions-1)
    - [`v : V -> U`](#v--v---u)
    - [`is_v : U -> Bool`](#is_v--u---bool)
    - [`as_v : U -> V`](#as_v--u---v)
    - [`mod_v : (V -> V) -> U -> U`](#mod_v--v---v---u---u)
  - [Modules and import statements](#modules-and-import-statements)
  - [Namespaces and overloading](#namespaces-and-overloading)
  - [More on import statements: filtering entities](#more-on-import-statements-filtering-entities)
  - [Recursion](#recursion)
  - [`eval` syntax](#eval-syntax)
  - [Type annotation](#type-annotation)
  - [Pattern matching](#pattern-matching)
  - [Traits](#traits)
  - [Associated types](#associated-types)
  - [Trait alias](#trait-alias)
  - [Type alias](#type-alias)
  - [Monads](#monads)
    - [What is monad?](#what-is-monad)
      - [State-like monads](#state-like-monads)
      - [Result-like monads](#result-like-monads)
      - [List-like monads](#list-like-monads)
    - [`do` block and monadic bind operator `*`](#do-block-and-monadic-bind-operator-)
    - [Chaining IO actions by `eval` and `forget`](#chaining-io-actions-by-eval-and-forget)
  - [Boxed and unboxed types](#boxed-and-unboxed-types)
    - [Functions](#functions)
    - [Tuples and unit](#tuples-and-unit)
    - [Array](#array)
    - [Structs](#structs-2)
    - [Unions](#unions-2)
  - [Foreign function interface (FFI)](#foreign-function-interface-ffi)
    - [Call a foreign function in Fix](#call-a-foreign-function-in-fix)
    - [Export a Fix value or function to a foreign language](#export-a-fix-value-or-function-to-a-foreign-language)
    - [Managing a foreign resource in Fix](#managing-a-foreign-resource-in-fix)
    - [Managing ownership of Fix's boxed value in a foreign language](#managing-ownership-of-fixs-boxed-value-in-a-foreign-language)
    - [Accessing fields of Fix's struct value from C](#accessing-fields-of-fixs-struct-value-from-c)
- [Operators](#operators)
- [Compiler feature](#compiler-feature)
  - [Fix projects and dependencies](#fix-projects-and-dependencies)
  - [Language Server Protocol](#language-server-protocol)
  - [Debugging](#debugging)


# Tutorial

## An example program

The following is a Fix program that calculates the first 30 numbers of Fibonacci sequence. 

```
module Main;

calc_fib : I64 -> Array I64;
calc_fib = |n| (
    let arr = Array::fill(n, 0);
    let arr = arr.set(0, 1);
    let arr = arr.set(1, 1);
    let arr = loop((2, arr), |(idx, arr)|
        if idx == arr.get_size {
            break $ arr
        } else {
            let x = arr.@(idx-1);
            let y = arr.@(idx-2);
            let arr = arr.set(idx, x+y);
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
[Run in playground](https://tttmmmyyyy.github.io/fixlang-playground/index.html?src2=bW9kdWxlIE1haW47DQoNCmNhbGNfZmliIDogSTY0IC0%2BIEFycmF5IEk2NDsNCmNhbGNfZmliID0gfG58ICgNCiAgICBsZXQgYXJyID0gQXJyYXk6OmZpbGwobiwgMCk7DQogICAgbGV0IGFyciA9IGFyci5zZXQoMCwgMSk7DQogICAgbGV0IGFyciA9IGFyci5zZXQoMSwgMSk7DQogICAgbGV0IGFyciA9IGxvb3AoKDIsIGFyciksIHwoaWR4LCBhcnIpfA0KICAgICAgICBpZiBpZHggPT0gYXJyLmdldF9zaXplIHsNCiAgICAgICAgICAgIGJyZWFrICQgYXJyDQogICAgICAgIH0gZWxzZSB7DQogICAgICAgICAgICBsZXQgeCA9IGFyci5AKGlkeC0xKTsNCiAgICAgICAgICAgIGxldCB5ID0gYXJyLkAoaWR4LTIpOw0KICAgICAgICAgICAgbGV0IGFyciA9IGFyci5zZXQoaWR4LCB4K3kpOw0KICAgICAgICAgICAgY29udGludWUgJCAoaWR4KzEsIGFycikNCiAgICAgICAgfQ0KICAgICk7DQogICAgYXJyDQopOw0KDQptYWluIDogSU8gKCk7DQptYWluID0gKA0KICAgIGxldCBmaWIgPSBjYWxjX2ZpYigzMCk7DQogICAgcHJpbnRsbiAkIEl0ZXJhdG9yOjpmcm9tX2FycmF5KGZpYikubWFwKHRvX3N0cmluZykuam9pbigiLCAiKQ0KKTs%3D)

If you save the above program to a file "main.fix" and run `fix run -f main.fix`, it prints 

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

In Fix, values, functions, types and traits defined in a source file is collected to a module. Each source file has to declare the name of the module it defines by `module {module_name};`. 

When Fix program runs, it calls `main` function defined in the `Main` module.

The usefulness of modules is hard to see in this example. They are useful when you construct a program from multiple source files.

A module name must starts with a capital letter.
Moreover, you can use a sequence of such strings concatenated by periods (e.g. `Main.Model.Impl`) as a module name. 
This grammar will be useful to express the hierarchy of modules.

## Global values

The following parts are definitions of two global values `calc_fib` and `main`.

```
calc_fib : I64 -> Array I64;
calc_fib = ...{expression A}...;

main : IO ();
main = ...{expression B}...;
```

These lines means that:

- `calc_fib` global value has type `I64 -> Array I64` and its value is defined by expression A.
- `main` global value has type `IO ()` and its value is defined by expression B.

In Fix, you have to specify the type of a global value explicitly. 

## Namespaces

The `Array` in `Array::fill` or `Iterator` in `Iterator::from_array` are namespaces. Namespace is the "address" of a name and used to distinguish two values (or types or traits, anything you define globally) with the same name.

Namespaces of a name can be omitted if the value specified by the name is unique, or can be inferred from the context. In fact, you can write simply `fill(n, 0)` instead of `Array::fill(n, 0)` because there is only one function named `fill` at the current version of standard library. The reasons I wrote `Array::fill(n, 0)` here are:

- `Array::fill(n, 0)` is more readable than `fill(n, 0)`, because it expresses that `fill` function is related to `Array` type. A reader may be able to infer that `Array::fill` will generate an array of specified length filled by a specified initial value.
- In the future, another function named `fill` may be added to a namespace other than `Array`. After that, the name `fill` may become ambiguous and the compile of the example program may start to fail.

Actually, the full name of `fill` is not `Array::fill` but `Std::Array::fill`. `Std` is a module to put entities provided by standard library. Module is nothing but a top-level namespace. The namespace `Array` is defined as the sub-namespace of `Std` and used to put functions related to arrays. Similarly, full name of `calc_fib` function is `Main::calc_fib`. You can omit (possibly full) prefix of namespaces of a name as long as the value referred to is uniquely inferred by compiler from the context.

## Types

Each value in Fix has its type. You can consider that a type is a set in mathematics, and value in Fix is an element of its type. 

The followings are examples of types:

- `I64`: the type of 64-bit signed integers.
- `Bool`: the type of boolean values (i.e., `true` and `false`).
- `Array a`: the type of arrays whose elements have type `a`. `Array` is called a type constructor, because it generates types `Array I64` or `Array Bool` when applied to a type. `a` is called a type parameter.
- `String`: the type of strings.
- `I64 -> Array I64`: the type of functions that takes an integer and returns an array of integers.
- `()`: the unit type. This type has a single value which is also written as `()`. 
- `(a, b)`: the type of pairs of values of `a` and `b`, where `a` and `b` are type parameters.
- `IO a`: the type whose value corresponds to an I/O action such as printing a string, opening a file and reading its content, etc. The type variable `a` is for the type of values returned by the I/O action. For example, if an I/O action reads the standard input as a `String` (and if we assume it never fails), it should have type `IO String`.
- `IO ()`: the type of I/O actions which returns no value. It is the type of `main` function of Fix program.
- `I64 -> Bool -> Array Bool`: this is equivalent to `I64 -> (Bool -> Array Bool)`, that is, the type of functions that receives an integer and returns a function that converts a boolean value into a boolean array. As an example, a function that produces a boolean array from its length and initial value has this type. In Fix, there is no concept of "two-variable functions". A function in Fix is a (partial) function in mathematical sense: it converts an element of a set into an element of another set (or fails). The type of something like "two-variable functions" can be represented as `a -> b -> c` or `(a, b) -> c`.

In Fix, the name of a specific type (such as `I64` or `Bool`) or a type constructor (such as `Array`) must starts with a capital letter.
A type that starts with a lowercase letter is interpreted as a type parameter. 
Each type parameter will be instanciated to a specific type when the program is compiled.

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

will be compiled, but the name `x` in the right hand side of `let x = x + 3` is treated as the name `x` defined in the previous line (i.e., its value is `5`), not as the new one.

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
    let arr = arr.set(0, 1);
    let arr = arr.set(1, 1);
    let arr = loop((2, arr), |(idx, arr)|
        if idx == arr.get_size {
            break $ arr
        } else {
            let x = arr.@(idx-1);
            let y = arr.@(idx-2);
            let arr = arr.set(idx, x+y);
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
let arr = arr.set(0, 1);
let arr = arr.set(1, 1);
let arr = loop((2, arr), |(idx, arr)|
    if idx == arr.get_size {
        break $ arr
    } else {
        let x = arr.@(idx-1);
        let y = arr.@(idx-2);
        let arr = arr.set(idx, x+y);
        continue $ (idx+1, arr)
    }
);
arr;
```

## Operator `.` and `$`

The operator `.` is another way of applying function to a value. It is defined as `x.f == f(x)`.

The precedence of the operator `.` is lower than function application by parenthes. So, if a function `method` has a type `Param -> Obj -> Result`, then `obj.method(arg)` is interpreted as `obj.(method(arg)) == method(arg)(obj) == method(arg, obj)`, not as `(obj.method)(arg)`.

In the program of Fibonacci sequence, the followings are examples of use of operator `.`:

- `arr.get_size`: `get_size` is a function of type `Array a -> I64`, which returns the length of an array. Note that you should not write `arr.get_size()` as if you call a method of a class on an instance in other languages. Remembering syntax sugars `f() == f(())` and `x.f == f(x)`, you can desugar the expression `arr.get_size()` to `get_size((), arr)`, which raises an error because `get_size` takes only one argument.
- `arr.set(0, 1)`: `set` is a function of type `I64 -> a -> Array a -> Array a`, which updates an element of an array to the specified value. 
- `arr.@(idx-1)`: `@` is a function of type `I64 -> Array a -> a`, which returns the element at the specified index.

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
    if idx == arr.get_size {
        break $ arr
    } else {
        let x = arr.@(idx-1);
        let y = arr.@(idx-2);
        let arr = arr.set(idx, x+y);
        continue $ (idx+1, arr)
    }
);
```

The initial value of this loop is `(2, arr)`. The loop body takes a tuple `(idx, arr)`, that is, the index of an array to be updated next, and an array to store the Fibonacci sequence whose values are already right at indices 0, ..., idx-1. If `idx` is less than `arr.get_size`, it calculates the value of Fibonacci sequence at `idx`, stores it to `arr`, and returns `continue $ (idx+1, arr)` to proceed to the next step. If `idx` has reached to `arr.get_size`, it returns `break $ arr` to end the loop. The return value of the `loop` function is an array.

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
- `as_continue : LoopResult s b -> s`: extracts a value of type `s` from a `LoopResult` value if it is created by `continue`. If not, this function aborts the program.
- `as_break : LoopResult s b -> s`: extracts a value of type `b` from a `LoopResult` value if it is created by `break`. If not, this function aborts the program.

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
- `set_price : I64 -> Product -> Product` and `set_sold : Bool -> Product -> Product`
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

## Mutation in Fix and reference counter

Remember that an expression in Fix is only a sentence that describes a value. It is essentially the same as a mathematical expression such as "1 + cos(pi/5)^2". There is no concept of "changing the value of a variable" which is ubiquitous in usual languages. In short, all values in Fix are immutable. 
For example, consider

```
main = (
    let arr0 = Array::fill(100, 1);
    let arr1 = arr0.set(0, 2);
    println("arr0.@(0): " + arr0.@(0).to_string + ".")
);
```

The above prints `arr0.@(0): 1.`, not `2`. This is because `arr0.set(0, 2)` is merely an expression that says "an array which is almost identical to `arr0` but with the 0th element replaced by `2`", and it is NOT a command "update the 0th element of `arr0` to `2`". To realize this behavior, `set` function in the above program has to clone `arr0` before updating the 0th element of an array.

Now consider the implementation of `calc_fib`.

```
calc_fib : I64 -> Array I64;
calc_fib = |n| (
    let arr = Array::fill(n, 0);
    let arr = arr.set(0, 1);
    let arr = arr.set(1, 1);
    let arr = loop((2, arr), |(idx, arr)|
        if idx == arr.get_size {
            break $ arr
        } else {
            let x = arr.@(idx-1);
            let y = arr.@(idx-2);
            let arr = arr.set(idx, x+y);
            continue $ (idx+1, arr)
        }
    );
    arr
);
```

The optimum time complexity of calculating Fibonacci sequence of length N is O(N). 
But if Fix had cloned the array at `let arr = arr.set(idx, x+y);` in the loop, it takes O(N) time for each loop step and the total time complexity becomes O(N^2).

In fact, `set` in the above program doesn't clone the array and `calc_fib` works in O(N) time, as expected. 
This is because if the given array will no longer be used, `set` omits cloning and just updates the given array. 
Let's consider a program which 

```
main = (
    let arr0 = Array::fill(100, 1);
    let arr1 = arr0.set(0, 2);
    println("arr1.@(0): " + arr1.@(0).to_string + ".")
);
```

(Note that `println` prints the 0th element of `arr1`, not of `arr0`.)
In this program, the call of `set` is the last usage of `arr0`. 
In such a case, `set` updates the given array without cloning, because the mutation of `arr0` will never be observed.

Go back to the `calc_fib` function. 
At the line `let arr = arr.set(idx, x+y);`, the name `arr` is redefined and set as pointing to the new array returned by `set` function. 
This ensures that the old array given to `set` function will never be referenced after this line. So it is evident that `set` function doesn't need to clone the given array, and in fact it doesn't.

As a summary, since values in Fix are immutable, the `set : I64 -> a -> Array a -> Array a` function basically returns a new array with one element replaced, but it omits cloning an array if the array will not be used later.

Fix judges whether a value may be used later or not by it's *reference counter*. Fix assigns reference counters to all boxed values - values which are always allocated on heap memory, and referenced by names or struct fields by pointers. Fix tracks the number of references to a boxed value using reference counter. A value is called "unique" if the reference counter is one, and called "shared" if otherwise. For convenience, an unboxed value is considered to be always unique.

Using terminologies introduced above, the `set` function directly mutates the array if and only if it is unique.

In implementing algorithms which depends on mutating arrays in `O(1)` time, such as dynamic programming, passing an unique array to `set` is quite important.
How to assure that an array is unique?
As I mentioned above, if `arr.set(idx, v)` is the last usage of `arr`, then `arr` is unique at the call of `set`(*).
In particular, writing `let arr = arr.set(idx, v);` assures that `set` receives an unique array, because 
since the updated array has the same name which was given to the old array, the old array will never be used after the call of `set`.

(*): This statement is true only when the array is referenced by a single thread.

# Other topics on syntax

## Boolean values and literals

The boolean type is `Bool` and its literals are `true` and `false`.

## Numbers and literals

Types for numbers are `I8`, `I16`, `I32`, `I64` (signed integers), `U8`, `U16`, `U32`, `U64` (unsigned integers) and `F32`, `F64` (floating point values).

Syntax for number literals is: 
```
"-"? ~ ASCII_DIGIT+ ~ ("." ~ ASCII_DIGIT+)? ~ ( "e" ~ ("+" | "-")? ~ ASCII_DIGIT+ )?
```

Integer literals can also be written with a hexadecimal (`0x`), octal (`0o`), or binary (`0b`) prefix.
Syntax for these number literal is:
```
"-"? ~ "0x" ~ ASCII_HEX_DIGIT+
"-"? ~ "0o" ~ ('0'..'7')+
"-"? ~ "0b" ~ ('0'..'1')+
```

Note that literals for floating point values requires at least one digit before and after the decimal point. 
For example, `1.` or `.1` is not valid float literal (where it is valid in C) and you need to write `1.0` or `0.1` instead.

The defaut type for integer liteal is `I64`, and the one for floating point value literal is `F64`.
For other types of numbers, you need to specify its type explicitl, such as `127_U8` or `3.14_F32` .

## Strings and literals

The type for strings is `String`. String literals are enclosed in double quotation marks, such as `"Hello World!"`

## Arrays and literals

The type for arrays is `Array`. Array literals are enclosed in "[" and "]", and each elements are separated by ",", such as `[1, 2, 3]`.

## Unit and tuples

Textual names of tuples are `Tuple{N}` where `N` is a natural number (which can be 0). For example, `Tuple2 I64 Bool` is equivalent to `(I64, Bool)`.
The unit type `()` is in fact the tuple of length 0, i.e., `Tuple0`.

## Structs

If you define a struct named `S` with a field `f` of type `F`, the following methods are defined in the namespace `S`.

### `@f : S -> F`

Extract the value of a field from a struct value.

### `set_f : F -> S -> S`

Modify a struct value by inserting a value to a field.
This function clones the given struct value if it is shared.

### `mod_f : (F -> F) -> S -> S`

Modify a struct value by acting on a field value.
This function clones the given struct value if it is shared.
What is special about this function is that if you call `obj.mod_field(f)` when both of `obj` and `obj.@field` are unique, it is assured that `f` receives the field value which is unique. So `obj.mod_field(f)` is NOT equivalent to `let v = obj.@field; obj.set_field(f(v))`.

### `act_f : [f : Functor] (F -> f F) -> S -> f S`

Perform a functorial action on the field of a struct value.
Semantically, `s.act_f(a)` is equivalent to `a(s.@f).map(|f| s.set_f(f))`. 
What is special about `act_f` is that if you call `s.act_f(a)` when both of `s` and `s.@f` is unique, it is assured that `a` receives an unique value.
See also document for `Array::act`.

This is known as [Lens](https://hackage.haskell.org/package/lens-5.0.1/docs/Control-Lens-Combinators.html#t:Lens) in Haskell community.

## Unions

If you define a union named `U` with a variant `v` of type `V`, the following methods are defined in the namespace `U`.

### `v : V -> U`

Constructs a union value from a variant value.

### `is_v : U -> Bool`

Check if a union value is created as the specified variant.

### `as_v : U -> V`

Converts a union value into a variant value if it is created as the variant. If not so, this function aborts the program.

### `mod_v : (V -> V) -> U -> U`

Modify a union value by a function acting on a variant.
What is special about `mod_v` is that if you call `u.mod_v(a)` when both of `u` and the value stored in `u` is unique, then `a` receives an unique value.

## Modules and import statements

In Fix, all entities (global values, types, traits) defined in a source file is collected to form a module.
Each source file has to declare the name of the module by `module {module_name};`.
A module name must starts with a capital letter.
Module name is used as the top-level namespace of entities defined in a source file.

You can import other module by `import {module_name};`. As an example, consider a program consists of two source files:

`lib.fix`:
```
module Lib;

module_name : String;
module_name = "Lib";
```

`main.fix`:
```
module Main;

import Lib;

module_name : String;
module_name = "Main";

main : IO ();
main = (
    println $ "This program consists of two modules, `" + Lib::module_name + "` and `" + Main::module_name + "`."
);
```

If you put these two files in a same directory and execute `fix run -f main.fix lib.fix`, it prints: 

```
This program consists of two modules, `Lib` and `Main`.
```

There is one special module: `Std`. This is a module of built-in entities. `Std` module is implicitly imported from all modules and you don't need to write `import Std` explicitly.

There are also other convenient modules which is included in fix's compiler, such as `Debug` or `HashMap`. To import these modules, you need to write import statements explicitly, but no need for adding source files to arguments of `fix run` or `fix build` command.

## Namespaces and overloading

Entities (global values, types and traits) in Fix can be overloaded in the sense that they can have conflicting name. 
All entities must be distinguished uniquely by their full name (name and namespaces).
Module name is used as the top-level namespace of entities defined in a source file. 
In addition, you can create a namespace explicitly by `namespace TheNameSpace { ... }`.

A namespace must starts with a capital letter.

For example, consider the following program.

```
module Main;

namespace BooleanTruth {
    truth : Bool;
    truth = true;
}

namespace IntegralTruth {
    truth : I64;
    truth = 42;
}
```

Then there are two entities named `truth`: `Main::BooleanTruth::truth` and `Main::IntegralTruth::truth`.

If you omit a prefix of (or all of) the namespaces of an entity, Fix tries to infer its full name by type information obtained up to the point where the entity is used.
For example, the follwing program

```
module Main;

namespace BooleanTruth {
    truth : Bool;
    truth = true;
}

namespace IntegralTruth {
    truth : I64;
    truth = 42;
}

main : IO ();
main = (
    println $ truth.to_string
);
```

failes to compile, because Fix cannot infer which `truth` should be used. 
On the other hand, the program

```
module Main;

namespace BooleanTruth {
    truth : Bool;
    truth = true;
}

namespace IntegralTruth {
    truth : I64;
    truth = 42;
}

main : IO ();
main = (
    println $ (0 + truth).to_string
);
```
[Run in playground](https://tttmmmyyyy.github.io/fixlang-playground/index.html?src2=bW9kdWxlIE1haW47DQoNCm5hbWVzcGFjZSBCb29sZWFuVHJ1dGggew0KICAgIHRydXRoIDogQm9vbDsNCiAgICB0cnV0aCA9IHRydWU7DQp9DQoNCm5hbWVzcGFjZSBJbnRlZ3JhbFRydXRoIHsNCiAgICB0cnV0aCA6IEk2NDsNCiAgICB0cnV0aCA9IDQyOw0KfQ0KDQptYWluIDogSU8gKCk7DQptYWluID0gKA0KICAgIHByaW50bG4gJCAoMCArIHRydXRoKS50b19zdHJpbmcNCik7)

will compile because Fix can infer the type of `truth` by the fact that it can be added to `0` of type `I64`.

A module name can contain periods, e.g., `Main.Model.Impl`.
In this case, an entity whose full name is `Main.Model.Impl::truth` can be referred to as `Impl::truth` or `Model.Impl::truth`.

## More on import statements: filtering entities

By writing `module {module_name};`, all entities defined in a module are imported. 
It is also possible to import only certain entities, or exclude certain entities.

For example, in the following program, every entity in the module `Std` is implicitly imported.
In fact, three types `Std::IO`, `Std::Tuple0` (which is the textual name of `()`), `Std::String` and a symbol `Std::IO::println` from `Std` module are used.

```
module Main;

main : IO ();
main = println("Hello, World!");
```

To import only entities that are actually used, you need to import `Std` explicitly and write:

```
module Main;
import Std::{IO, Tuple0, String, IO::println};

main : IO ();
main = println("Hello, World!");
```

If you want to import `Std::IO::eprintln` in addition, you can write:

```
import Std::{IO, Tuple0, String, IO::println, IO::eprintln};
```

or

```
import Std::{IO, Tuple0, String, IO::{println, eprintln}};
```

If importing any entities in the `Std::IO` namespace is OK, you can write:

```
module Main;
import Std::{IO, Tuple0, String, IO::*};

main : IO ();
main = println("Hello, World!");
```

Let's see another example. 
The `Std` module provides a type `Tuple2`, whose value is constructed by writing `(x, y)`. 
Assume that you are defining and using your own `Tuple2`:

```
module Main;

type Tuple2 a b = struct { fst : a, snd : b };

impl [a : ToString, b : ToString] Tuple2 a b : ToString {
    to_string = |t| "(" + t.@fst.to_string + ", " + t.@snd.to_string + ")";
}

main : IO ();
main = println $ Tuple2 { fst : "Hello", snd : "World!" }.to_string;
```

The above code cannot be compiled because there are two types named as `Tuple2`.

```
error: Type name `Tuple2` is ambiguous. There are `Main::Tuple2`, `Std::Tuple2`.
```

Of course, you can also resolve this issue by adding `Main::` in front of each occurrence of `Tuple2`.
Another solution for this issue is importing `Std` explicitly and hiding `Tuple2`:

```
module Main;

import Std hiding Tuple2;

type Tuple2 a b = struct { fst : a, snd : b };

impl [a : ToString, b : ToString] Tuple2 a b : ToString {
    to_string = |t| "(" + t.@fst.to_string + ", " + t.@snd.to_string + ")";
}

main : IO ();
main = println $ Tuple2 { fst : "Hello", snd : "World!" }.to_string;
```

You can hide multiple entities by writing such as `import Std hiding {symbol0, Type1, Namespace2::*}`.

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
[Run in playground](https://tttmmmyyyy.github.io/fixlang-playground/index.html?src2=bW9kdWxlIE1haW47DQoNCmZpYiA6IEk2NCAtPiBJNjQ7DQpmaWIgPSB8bnwgKA0KICAgIGlmIG4gPT0gMCB7DQogICAgICAgIDANCiAgICB9IGVsc2UgaWYgbiA9PSAxIHsNCiAgICAgICAgMQ0KICAgIH0gZWxzZSB7DQogICAgICAgIGZpYihuLTEpICsgZmliKG4tMikNCiAgICB9DQopOw0KDQptYWluIDogSU8gKCk7DQptYWluID0gcHJpbnQgJCBmaWIoMzApLnRvX3N0cmluZzsgLy8gODMyMDQw)

On the other hand, Fix's `let`-binding doesn't allow to make recursive definition. To define a recursive function locally, use `fix` built-in function.

## `eval` syntax

An expression `eval {expression_0}; {expression_1}` evaluates both of `{expression_0}` and `{expression_1}`, and returns value of `{expression_1}`.
The type of `{expression_0}` has to be `()`.

Since Fix is functional, only evaluating an expression and ignoring the result has no effect in most cases. 
Typical use-cases of `eval` are to call functions which return `()` to get side-effects.

- Calling functions in `Debug` module, such as `assert : Lazy String -> Bool -> ()` or `debug_println : String -> ()`. 
- Calling C functions by FFI_CALL. 
- Sequentially calling I/O functions. 

Example: 

```
module Main;
import Debug;

main : IO ();
main = (
    eval assert(|_|"1 is not 2!", 1 == 2);
    eval "Contradiction: ".borrow_c_str(|ptr| let _ = FFI_CALL[I32 printf(Ptr, ...), ptr]; ());
    eval *println("1 is equal to 2!");
    pure()
);
```
[Run in playground](https://tttmmmyyyy.github.io/fixlang-playground/?src2=bW9kdWxlIE1haW47DQppbXBvcnQgRGVidWc7DQoNCm1haW4gOiBJTyAoKTsNCm1haW4gPSAoDQogICAgZXZhbCBhc3NlcnQofF98IjEgaXMgbm90IDIhIiwgMSA9PSAyKTsNCiAgICBldmFsICJDb250cmFkaWN0aW9uOiAiLmJvcnJvd19jX3N0cih8cHRyfCBDQUxMX0NbSTMyIHByaW50ZihQdHIsIC4uLiksIHB0cl0pOw0KICAgIGV2YWwgKnByaW50bG4oIjEgaXMgZXF1YWwgdG8gMiEiKTsNCiAgICBwdXJlKCkNCik7)

For detail of `*` operator in front of `print` and `println`, see [Monads](#monads). 
For FFI_CALL, see [Calling C functions from Fix](#calling-c-functions-from-fix).

## Type annotation

You need to write types of global value explicity. You can specify the type of a local value for readability or for helping type / namespace inference of Fix compiler.

The following demonstrates type annotations for local values.

```
module Main;

main : IO ();
main = (
    let x = 42 : I64; // Type annotation on expression.
    let y : I64 = 42; // Type annotation on let-binding.
    let f = |v : I64| v * 3; // Type annotation on a variable of function.
    
    eval *(println $ x.to_string);
    eval *(println $ y.to_string);
    eval *(println $ f(14).to_string);

    pure()
);
```
[Run in playground](https://tttmmmyyyy.github.io/fixlang-playground/index.html?src2=bW9kdWxlIE1haW47DQoNCm1haW4gOiBJTyAoKTsNCm1haW4gPSAoDQogICAgbGV0IHggPSA0MiA6IEk2NDsgLy8gVHlwZSBhbm5vdGF0aW9uIG9uIGV4cHJlc3Npb24uDQogICAgbGV0IHkgOiBJNjQgPSA0MjsgLy8gVHlwZSBhbm5vdGF0aW9uIG9uIGxldC1iaW5kaW5nLg0KICAgIGxldCBmID0gfHYgOiBJNjR8IHYgKiAzOyAvLyBUeXBlIGFubm90YXRpb24gb24gYSB2YXJpYWJsZSBvZiBmdW5jdGlvbi4NCiAgICANCiAgICBsZXQgXyA9ICoocHJpbnRsbiAkIHgudG9fc3RyaW5nKTsNCiAgICBsZXQgXyA9ICoocHJpbnRsbiAkIHkudG9fc3RyaW5nKTsNCiAgICBsZXQgXyA9ICoocHJpbnRsbiAkIGYoMTQpLnRvX3N0cmluZyk7DQoNCiAgICBwdXJlKCkNCik7)

## Pattern matching

Pattern matching are available in let-binding or function definition.

```
module Main;

type IntBool = struct { int_field : I64, bool_field : Bool };

destructure : IntBool -> (I64, Bool);
destructure = |IntBool { int_field : i, bool_field : b }| (i, b); // Pattern matching on function definition

main : IO ();
main = (
    let (i, b) = destructure $ IntBool { int_field : 42, bool_field : true }; // Pattern matching on let-binding
    println $ "(" + i.to_string + ", " + b.to_string + ")"
);
```
[Run in playground](https://tttmmmyyyy.github.io/fixlang-playground/index.html?src2=bW9kdWxlIE1haW47DQoNCnR5cGUgSW50Qm9vbCA9IHN0cnVjdCB7IGludF9maWVsZCA6IEk2NCwgYm9vbF9maWVsZCA6IEJvb2wgfTsNCg0KZGVzdHJ1Y3R1cmUgOiBJbnRCb29sIC0%2BIChJNjQsIEJvb2wpOw0KZGVzdHJ1Y3R1cmUgPSB8SW50Qm9vbCB7IGludF9maWVsZCA6IGksIGJvb2xfZmllbGQgOiBiIH18IChpLCBiKTsgLy8gUGF0dGVybiBtYXRjaGluZyBvbiBmdW5jdGlvbiBkZWZpbml0aW9uDQoNCm1haW4gOiBJTyAoKTsNCm1haW4gPSAoDQogICAgbGV0IChpLCBiKSA9IGRlc3RydWN0dXJlICQgSW50Qm9vbCB7IGludF9maWVsZCA6IDQyLCBib29sX2ZpZWxkIDogdHJ1ZSB9OyAvLyBQYXR0ZXJuIG1hdGNoaW5nIG9uIGxldC1iaW5kaW5nDQogICAgcHJpbnRsbiAkICIoIiArIGkudG9fc3RyaW5nICsgIiwgIiArIGIudG9fc3RyaW5nICsgIikiDQopOw%3D%3D)

## Traits

A Trait is a set of types. 
A trait is defined by a set of "methods" to be implemented by each member of it.

```
module Main;

// A Trait is a set of types. 
// A trait is defined by a set of "methods" to be implemented by each member of it.

// `Greeter` is a set of types, where...
trait a : Greeter {
    // whose member has a method `greeting` that converts a value of type `a` into a greeting message greeting.
    greeting : a -> String;
}

// Let `I64` belong to the trait `MyToString`, where 
impl I64 : Greeter {
    // the `greeting` method is defined as follows.
    greeting = |n| "Hi! I'm a 64-bit integer " + n.to_string + "!";
}

/*
Traits are used for overloading operators.
For example, `Eq` trait is defined in standard library as follows: 

```
trait a : Eq {
    eq : a -> a -> Bool
}
```

Each expression `x == y` is a syntax suger for `Eq::eq(x, y)`.
*/

// As another example, 
type Pair a b = struct { fst: a, snd: b };

// In the trait implementation, you can specify constraints on type variables in `[]` bracket after `impl`.
impl [a : Eq, b : Eq] Pair a b : Eq {
    eq = |lhs, rhs| (
        lhs.@fst == rhs.@fst && lhs.@snd == rhs.@snd
    );
}

// You can specify constraints on type variables in the `[]` bracket before a type signature.
search : [a : Eq] a -> Array a -> I64;
search = |elem, arr| loop(0, |idx|
    if idx == arr.get_size { break $ -1 };
    if arr.@(idx) == elem { break $ idx };
    continue $ (idx + 1)
);

// An example of defining higher-kinded trait.
// All type variable has kind `*` by default, and any kind of higher-kinded type variable need to be annoted explicitly.
trait [f : *->*] f : MyFunctor {
    mymap : (a -> b) -> f a -> f b;
}

// An example of implementing higher-kinded trait.
// `Array` is a type of kind `* -> *`, so matches to the kind of trait `MyFunctor`.
impl Array : MyFunctor {
    mymap = |f, arr| (
        Array::from_map(arr.get_size, |idx| f(arr.@(idx)))
    );
}

main : IO ();
main = (
    let arr = Array::from_map(6, |x| x); // arr = [0,1,2,...,9].
    let arr = arr.mymap(|x| Pair { fst: x % 2, snd: x % 3 }); // arr = [(0, 0), (1, 1), (0, 2), ...].
    let x = arr.search(Pair { fst: 1, snd: 2}); // 5, the first number x such that x % 2 == 1 and x % 3 == 2.
    println $ x.greeting // This should print "Hi! I'm a 64-bit integer 5!".
);
```
[Run in playground](https://tttmmmyyyy.github.io/fixlang-playground/index.html?src2=bW9kdWxlIE1haW47DQoNCi8vIEEgVHJhaXQgaXMgYSBzZXQgb2YgdHlwZXMuIA0KLy8gQSB0cmFpdCBpcyBkZWZpbmVkIGJ5IGEgc2V0IG9mICJtZXRob2RzIiB0byBiZSBpbXBsZW1lbnRlZCBieSBlYWNoIG1lbWJlciBvZiBpdC4NCg0KLy8gYEdyZWV0ZXJgIGlzIGEgc2V0IG9mIHR5cGVzLCB3aGVyZS4uLg0KdHJhaXQgYSA6IEdyZWV0ZXIgew0KICAgIC8vIHdob3NlIG1lbWJlciBoYXMgYSBtZXRob2QgYGdyZWV0aW5nYCB0aGF0IGNvbnZlcnRzIGEgdmFsdWUgb2YgdHlwZSBgYWAgaW50byBhIGdyZWV0aW5nIG1lc3NhZ2UgZ3JlZXRpbmcuDQogICAgZ3JlZXRpbmcgOiBhIC0%2BIFN0cmluZzsNCn0NCg0KLy8gTGV0IGBJNjRgIGJlbG9uZyB0byB0aGUgdHJhaXQgYE15VG9TdHJpbmdgLCB3aGVyZSANCmltcGwgSTY0IDogR3JlZXRlciB7DQogICAgLy8gdGhlIGBncmVldGluZ2AgbWV0aG9kIGlzIGRlZmluZWQgYXMgZm9sbG93cy4NCiAgICBncmVldGluZyA9IHxufCAiSGkhIEknbSBhIDY0LWJpdCBpbnRlZ2VyICIgKyBuLnRvX3N0cmluZyArICIhIjsNCn0NCg0KLyoNClRyYWl0cyBhcmUgdXNlZCBmb3Igb3ZlcmxvYWRpbmcgb3BlcmF0b3JzLg0KRm9yIGV4YW1wbGUsIGBFcWAgdHJhaXQgaXMgZGVmaW5lZCBpbiBzdGFuZGFyZCBsaWJyYXJ5IGFzIGZvbGxvd3M6IA0KDQpgYGANCnRyYWl0IGEgOiBFcSB7DQogICAgZXEgOiBhIC0%2BIGEgLT4gQm9vbA0KfQ0KYGBgDQoNCkVhY2ggZXhwcmVzc2lvbiBgeCA9PSB5YCBpcyBhIHN5bnRheCBzdWdlciBmb3IgYEVxOjplcSh4LCB5KWAuDQoqLw0KDQovLyBBcyBhbm90aGVyIGV4YW1wbGUsIA0KdHlwZSBQYWlyIGEgYiA9IHN0cnVjdCB7IGZzdDogYSwgc25kOiBiIH07DQoNCi8vIEluIHRoZSB0cmFpdCBpbXBsZW1lbnRhdGlvbiwgeW91IGNhbiBzcGVjaWZ5IGNvbnN0cmFpbnRzIG9uIHR5cGUgdmFyaWFibGVzIGluIGBbXWAgYnJhY2tldCBhZnRlciBgaW1wbGAuDQppbXBsIFthIDogRXEsIGIgOiBFcV0gUGFpciBhIGIgOiBFcSB7DQogICAgZXEgPSB8bGhzLCByaHN8ICgNCiAgICAgICAgbGhzLkBmc3QgPT0gcmhzLkBmc3QgJiYgbGhzLkBzbmQgPT0gcmhzLkBzbmQNCiAgICApOw0KfQ0KDQovLyBZb3UgY2FuIHNwZWNpZnkgY29uc3RyYWludHMgb24gdHlwZSB2YXJpYWJsZXMgaW4gdGhlIGBbXWAgYnJhY2tldCBiZWZvcmUgYSB0eXBlIHNpZ25hdHVyZS4NCnNlYXJjaCA6IFthIDogRXFdIGEgLT4gQXJyYXkgYSAtPiBJNjQ7DQpzZWFyY2ggPSB8ZWxlbSwgYXJyfCBsb29wKDAsIHxpZHh8DQogICAgaWYgaWR4ID09IGFyci5nZXRfc2l6ZSB7IGJyZWFrICQgLTEgfTsNCiAgICBpZiBhcnIuQChpZHgpID09IGVsZW0geyBicmVhayAkIGlkeCB9Ow0KICAgIGNvbnRpbnVlICQgKGlkeCArIDEpDQopOw0KDQovLyBBbiBleGFtcGxlIG9mIGRlZmluaW5nIGhpZ2hlci1raW5kZWQgdHJhaXQuDQovLyBBbGwgdHlwZSB2YXJpYWJsZSBoYXMga2luZCBgKmAgYnkgZGVmYXVsdCwgYW5kIGFueSBraW5kIG9mIGhpZ2hlci1raW5kZWQgdHlwZSB2YXJpYWJsZSBuZWVkIHRvIGJlIGFubm90ZWQgZXhwbGljaXRseS4NCnRyYWl0IFtmIDogKi0%2BKl0gZiA6IE15RnVuY3RvciB7DQogICAgbXltYXAgOiAoYSAtPiBiKSAtPiBmIGEgLT4gZiBiOw0KfQ0KDQovLyBBbiBleGFtcGxlIG9mIGltcGxlbWVudGluZyBoaWdoZXIta2luZGVkIHRyYWl0Lg0KLy8gYEFycmF5YCBpcyBhIHR5cGUgb2Yga2luZCBgKiAtPiAqYCwgc28gbWF0Y2hlcyB0byB0aGUga2luZCBvZiB0cmFpdCBgTXlGdW5jdG9yYC4NCmltcGwgQXJyYXkgOiBNeUZ1bmN0b3Igew0KICAgIG15bWFwID0gfGYsIGFycnwgKA0KICAgICAgICBBcnJheTo6ZnJvbV9tYXAoYXJyLmdldF9zaXplLCB8aWR4fCBmKGFyci5AKGlkeCkpKQ0KICAgICk7DQp9DQoNCm1haW4gOiBJTyAoKTsNCm1haW4gPSAoDQogICAgbGV0IGFyciA9IEFycmF5Ojpmcm9tX21hcCg2LCB8eHwgeCk7IC8vIGFyciA9IFswLDEsMiwuLi4sOV0uDQogICAgbGV0IGFyciA9IGFyci5teW1hcCh8eHwgUGFpciB7IGZzdDogeCAlIDIsIHNuZDogeCAlIDMgfSk7IC8vIGFyciA9IFsoMCwgMCksICgxLCAxKSwgKDAsIDIpLCAuLi5dLg0KICAgIGxldCB4ID0gYXJyLnNlYXJjaChQYWlyIHsgZnN0OiAxLCBzbmQ6IDJ9KTsgLy8gNSwgdGhlIGZpcnN0IG51bWJlciB4IHN1Y2ggdGhhdCB4ICUgMiA9PSAxIGFuZCB4ICUgMyA9PSAyLg0KICAgIHByaW50bG4gJCB4LmdyZWV0aW5nIC8vIFRoaXMgc2hvdWxkIHByaW50ICJIaSEgSSdtIGEgNjQtYml0IGludGVnZXIgNSEiLg0KKTs%3D)

## Associated types

An associated type is a type level function whose domain is (the set of members of) a trait.

```
module Main;
import Debug;

// A trait for collection-like types.
trait c : Collects {
    // Associated type definition.
    // Here, we define a type level function `Elem` which domain is the trait `Collects` and returns a type.
    // When implementing `Collects` trait for a type `c`, you need to specify the value (which is a type) `Elem c`.
    type Elem c;
    // You can use `Elem c` in trait method definitions.
    empty : Elem c;
    insert : Elem c -> c -> c;
    to_iter : c -> Iterator (Elem c);
}

// Implementing `Collects` for arrays.
impl Array a : Collects {
    type Elem (Array a) = a;
    empty = [];
    insert = |x, xs| xs.push_back(x);
    to_iter = |xs| Array::to_iter(xs);
}

// Implementing `Collects` for iterators.
impl Iterator a : Collects {
    type Elem (Iterator a) = a;
    empty = Iterator::empty;
    insert = |x, xs| xs.push_front(x);
    to_iter = |xs| xs;
}

// Takes two collections with the same element type, and insert all elements of the second collection to the first one.
extend : [c1 : Collects, c2 : Collects, Elem c1 = e, Elem c2 = e] c1 -> c2 -> c2;
extend = |xs, ys| xs.to_iter.fold(ys, |ys, x| ys.insert(x));

// Take a collection whose element type implements `ToString` trait, and stringify the collection.
stringify : [c : Collects, Elem c = e, e : ToString] c -> String;
stringify = |xs| xs.to_iter.map(to_string).join(", ");

main : IO ();
main = (
    eval assert_eq(|_|"", [1, 2, 3].extend([4, 5, 6]).stringify, "1, 2, 3, 4, 5, 6");
    pure()
);
```

Associated type can have higher arity. 
The following is an example of defining type level function of arity 2 using associated types.

```
module Main;
import Debug;

// We define addition on type level numbers using associated types.

// First, we prepare type level numbers.
type Zero = unbox struct { data : () };
type Succ n = unbox struct { data : () };
type One = Succ Zero;
type Two = Succ One;
type Three = Succ Two;

// `Value` is a type which is parametrized by a type level number and holds a value of it.
type Value n = unbox struct { data : I64 };

// Define the trait for type level numbers, which requires
// - an associated type `Add` which performs addition of two type level numbers,
// - a value of type `Value n` which holds a value of the type level number.
trait n : Nat {
    type Add n m; // An associated type of arity 2.
    value : Value n;
}

// Implement `Nat` for type level by induction.
impl Zero : Nat {
    type Add Zero m = m;
    value = Value { data : 0 };
}
impl [n : Nat] Succ n : Nat {
    type Add (Succ n) m = Succ (Add n m);
    value = (
        // The following is how we extract a value from a type level number:
        // We select the appropriate implementation of the trait method `Nat::value` using type annotation.
        let n = (Nat::value : Value n).@data;
        Value { data : n + 1 }
    );
}

main : IO ();
main = (
    eval assert_eq(|_|"", (Nat::value : Value Zero).@data, 0);
    eval assert_eq(|_|"", (Nat::value : Value One).@data, 1);
    eval assert_eq(|_|"", (Nat::value : Value Two).@data, 2);
    eval assert_eq(|_|"", (Nat::value : Value (Add One Two)).@data, 3);
    pure()
);
```

## Trait alias

You can define an alias of traits. Defining a trait alias by 

```
trait Foo = Bar + Baz;
```

allows you to write `a : Foo` instead of `a : Bar, a : Baz`.

You cannot implement a trait alias directly. If you want to implement `Foo` for a type `SomeType`, then implement `SomeType : Bar` and `SomeType : Baz` individually.

## Type alias

You can define type alias as follows:

```
type Name = String;
```

Type alias does NOT define a new type: it is merely another name of the aliased type.

You can also define higher-kinded type alias. The following is an example of such type alias defined in "Std":

```
type Lazy a = () -> a;
```

which defines a type alias `Lazy` of kind `* -> *`.

## Monads

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

In Fix's standard library, `Iterator` is an example of list-like monad. For list-like moads, `[x, y, z, ...].bind(f)` represents `f(x) + f(y) + f(z) + ...`, where `+` concatenates two list-like values. `bind` is sometimes called "flatMap" in other languages.

`pure(x)` represents an singleton value `[x]`. 

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

### Chaining IO actions by `eval` and `forget`

The `println : String -> IO ()` function takes a string and returns an IO action which prints the string to the standard output.
If you want to perform `println` multiple times, you can write as follows using operator `*`.

```
module Main;

main : IO ();
main = (
    let _ = *println("The sum of 1 + 2 is: ");
    let _ = *println((1 + 2).to_string);
    pure()
);
```

Here, `pure() : IO ()` is an IO action which does nothing and just returns `()`. 
Since we don't need the result of the IO action `print(...)`, we get the result by a variable named `_` and forget about it.

[The `eval` syntax](#eval-syntax) is a shorter way to write `let _ = `. Using this syntax, the following code can be rewritten as follows.

```
module Main;

main : IO ();
main = (
    eval *println("The sum of 1 + 2 is: ");
    eval *println((1 + 2).to_string);
    pure()
);
```

Actually, `eval` only accepts an expression of type `()`. It is a syntax sugar of `let _ : () = `. 
This prevents you writing wrongly as 

```
module Main;

main : IO ();
main = (
    eval *println("The sum of 1 + 2 is: ");
    eval println((1 + 2).to_string); // Type error!
    pure()
);
```

because `println((1 + 2).to_string) : IO ()` does not match to the type `()`.

Assume that you have `read : IO String` which reads strings from standard input, and write:

```
module Main;

read : IO String;
read = read_content(stdin).map(as_ok);

main : IO ();
main = (
    eval *read; // Type error!
    eval *println("You can type!");
    pure()
);
```

Fix failes to compile this, because the type of `*read` is not `()`, but `String`. 
In this case, the function `Std::Functor::forget : [f : Functor] f a -> f ()` will be useful. 
When used with `IO` monad, this function converts an `IO a` to `IO ()` by forgetting the result value.
Using `forget`, the above can be rewritten as 

```
module Main;

read : IO String;
read = read_content(stdin).map(as_ok);

main : IO ();
main = (
    eval *read.forget; // `forget : IO String -> IO ()`
    eval *println("You can type!");
    pure()
);
```

which can be compiled successfully. 
It is even better because it expresses we throw away the result of `read` intentionally.

## Boxed and unboxed types

Types in Fix are divided into boxed types and unboxed types. Boxed types and unboxed types are similar to things called as "reference types" and "value types" in other languages, respectively.

* Value of boxed types are allocated in heap memory. Local names and struct / union fields whose types are boxed are compiled as pointers to the values. 
* Values of unboxed types are directly embedded into the stack memory, structs and unions. 

In general, types that contain a lot of data (such as `Array`) are suited to be boxed because boxed types have lower copying costs. On the other hand, types containing small data (such as `I64`) can be unboxed to reduce the cost of increasing or decreasing the reference counter.

### Functions

Functions are unboxed, but captured values are stored to an unnamed boxed struct.

### Tuples and unit

Tuple types are unboxed, because tuple is intended to have only a few fields. If you want to use many fields, you should define a new struct.
Tuples are special forms of structs whose field names are `0`, `1`, `2`, etc. 

Since the unit type is a tuple type of length 0, the unit type is also unboxed.

### Array

`Std::Array` is a boxed type.

### Structs

Structs are unboxed by default. To define boxed struct type, write `box` specifier before `struct`.

Example:
```
type Product = box struct { price: I64, sold: Bool };
```

### Unions

Unions are unboxed by default. To define boxed union type, write `box` specifier before `struct`.

```
type Weight = box union { pound: I64, kilograms: I64 };
```

## Foreign function interface (FFI)

You can link a static or shared library to a Fix program by `--static-link` (`-s`) or `--dynamic-link` (`-s`) compiler flag, and call native functions in the Fix program or call Fix functions in the library.

Note that using FFI can easily break Fix's assurance such as immutability or memory safety.
The programmer has a responsibility to hide the side effect of a foreign function into `IO`, and manage resources properly to avoid segmentation fault or memory leak.

### Call a foreign function in Fix

Use the `FFI_CALL` expression to call a foreign function in Fix. The syntax is as follows:

```
FFI_CALL[{function_signature}, {arg_0}, {arg_1}, ...]
```

Example: 

```
main : IO ();
main = (
    eval "Hello C function!\n".borrow_c_str(|ptr|
        let _ = FFI_CALL[I32 printf(Ptr), ptr]; // Explicitly ignore the result of `printf`.
        ()
    );
    pure()
);
```

In `{c_function_signature}`, you need to specify the name and the signature of the foreign function to call.
The signature should be written in the form of `{return_type} {function_name}({arg_type_0}, {arg_type_1}, ...)`.
For `{return_type}` or `{arg_type_i}`, you can use the following types:

- `Ptr` for pointers.
- `I8`, `U8`, `I16`, `U16`, `I32`, `U32`, `I64`, `U64`, `F32`, `F64` for primitive numeric types.
- `CChar`, `CUnsignedChar`, `CShort`, `CUnsignedShort`, `CInt`, `CUnsignedInt`, `CLong`, `CUnsignedLong`, `CLongLong`, `CUnsignedLongLong`, `CSizeT`, `CFloat`, `CDouble` for C's primitive numeric types.
- `()` instead of `void` for a function that returns nothing.

### Export a Fix value or function to a foreign language

You can export a value of Fix using `FFI_EXPORT[{fix_value_name}, {c_function_name}];` to make it available from a foreign language.

```
fix_increment : CInt -> CInt;
fix_increment = |x| x + 1.to_CInt;
FFI_EXPORT[fix_increment, increment]; // Define a function `int increment(int)`.
```

If the foreign language is C, you should declare `int increment(int);` and call it to use `fix_increment` in the C program.

The signature of the exported function is automatically determined by the type of the exported Fix value, as demonstrated in the following code:

```
x : CInt; 
FFI_EXPORT[x, f]; // int f(void);

x : CInt -> CInt;
FFI_EXPORT[x, f]; // int f(int);

x : CInt -> CInt;
FFI_EXPORT[x, f]; // int f(int);

x : IO ();
FFI_EXPORT[x, f]; // void f(void);

x : IO CInt;
FFI_EXPORT[x, f]; // int f(void);

x : CInt -> IO CInt;
FFI_EXPORT[x, f]; // int f(int);
```

### Managing a foreign resource in Fix

Some C functions allocate a resource which should be deallocated by another C function in the end. 
Most famous examples may be `malloc` / `free` and `fopen` / `fclose`.

If you allocate a resource using `FFI_CALL`, then you need to call the deallocation function again by `FFI_CALL` at the end of the resource's lifetime.

A useful way to manage the resource properly is to use `Std::FFI::Destructor` type.
This is a boxed wrapper type of a specified type associated with a deallocation function.
When a value of `Destructor` type is dropped, the deallocation function is called automatically.
So you can use `Destructor` to wrap a "handle" to a foreign resource and to call the deallocation function automatically when the `Destructor` value is no longer used in Fix code.

For details, see [the document for `Destructor`](./BuiltinLibraries.md#type-destructor).

### Managing ownership of Fix's boxed value in a foreign language

The function `Std::FFI::unsafe_get_retained_ptr_of_boxed_value : a -> Ptr` returns a retained pointer to a *boxed* value allocated by Fix.
Here, "retained" means that the pointer has a shared ownership of the value, and you are responsible for decrementing the reference counter to avoid memory leak.
You can get back a Fix value from a retained pointer by `Std::FFI::unsafe_get_boxed_value_from_retained_ptr : Ptr -> a`.

If you have a retained pointer of a Fix value in a foreign language, you may need to release it (i.e., decrement the reference counter) when you drop the pointer, or retain it (i.e., increment the reference counter) when you copy the pointer.
To do this, first get the pointer to the retain / release function for a Fix value by `Std::FFI::unsafe_get_release_function_of_boxed_value` and `Std::FFI::unsafe_get_retain_function_of_boxed_value`:

- `Std::FFI::unsafe_get_release_function_of_boxed_value : a -> Ptr`
- `Std::FFI::unsafe_get_retain_function_of_boxed_value : a -> Ptr`

Each function returns a function pointer of type `void (*)(void*)`.
Then you can retain / release a Fix's value of type `a` via the function pointer.

NOTE:
Fix's reference counting is not thread-safe by default. 
So if you get a pointer to a Fix's boxed value and share it between multiple threads, then retaining / releasing the pointer in the way described above may cause data race.

To avoid this, first add the `--threaded` compiler flag.
Moreover, call `Std::mark_threaded : a -> a` on the boxed value before obtaining the pointer.
The `Std::mark_threaded` function traverses all values reachable from the given value, and changes them into multi-threaded mode so that the reference counting on them will be done in thread-safe manner.

### Accessing fields of Fix's struct value from C

Assume that you have a *boxed* struct type
```
type Vec = box struct { x : CDouble, y : CDouble };
```
and a C program
```
struct Vec {
    double x;
    double y;
}

void access_vec(Vec* v) {
    // Do something with / to `v->x` and `v->y`.
}
```

If you want to access to the fields `x` and `y` of Fix's object `vec` from C side, `Std::FFI::unsafe_borrow_boxed_data_ptr : (Ptr -> b) -> a -> b` will be useful: 
`vec.unsafe_borrow_boxed_data_ptr(|p| FFI_CALL[() access_vec(Ptr), p])` will allows `access_vec` on work on `vec.@x` and `vec.@y`.

NOTE: 
At least in the current version of Fix, the memory layout of Fix's struct is determined by the default behaviour of LLVM, and as long as I know it is equivalent to C's struct memory layout. 
In a future version, the situation may be changed. I may introduce a specifier (suppose it is written as `expr_c`) for a programmer to assure that the layout is equivalent to C, and the struct layout with no `expr_c` specifier may be optimized (e.g., reorder field ordering).

# Operators

The following is the table of operators sorted by its precedence (operator of higher precedence appears earlier).

| Operator / syntax | Type                     | Trait / function                    | Explanation                                                        |
| ----------------- | ------------------------ | ----------------------------------- | ------------------------------------------------------------------ |
| f(x)              | syntax                   | -                                   | function application                                               |
| .                 | left associative binary  | -                                   | right-to-left function application: x.f = f(x)                     |
| *                 | unary prefix             | Std::Monad / bind                   | monadic bind                                                       |
| <<                | left associative binary  | Std::compose                        | right-to-left function composition: g << f = &#124;x&#124; g(f(x)) |
| >>                | left associative binary  | Std::compose                        | left-to-right function composition: f >> g = &#124;x&#124; g(f(x)) |
| - (minus sign)    | unary prefix             | Std::Neg / neg                      | negative of number                                                 |
| !                 | unary prefix             | Std::Not / not                      | logical NOT                                                        |
| *                 | left associative binary  | Std::Mul / mul                      | multiplication of numbers                                          |
| /                 | left associative binary  | Std::Div / div                      | division of numbers                                                |
| %                 | left associative binary  | Std::Rem / rem                      | reminder of division                                               |
| +                 | left associative binary  | Std::Add / add                      | addition of numbers                                                |
| - (minus sign)    | left associative binary  | Std::Sub / sub                      | subtraction of numbers                                             |
| ==                | left associative binary  | Std::Eq / eq                        | equality comparison                                                |
| !=                | left associative binary  | -                                   | `x != y` is interpreted as `!(x == y)`                             |
| <=                | left associative binary  | Std::LessThanOrEq / less_than_or_eq | less-than-or-equal-to comparison                                   |
| >=                | left associative binary  | -                                   | `x >= y` is interpreted as `y <= x`                                |
| <                 | left associative binary  | Std::LessThan / less_than           | less-than comparison                                               |
| >                 | left associative binary  | -                                   | `x > y` is interpreted as `y < x`                                  |
| &&                | right associative binary | -                                   | short-circuit logical AND.                                         |
| &#124;&#124;      | right associative binary | -                                   | short-circuit logical OR                                           |
| $                 | right associative binary | -                                   | right associative function application: f $ g $ x = f(g(x))        |

# Compiler feature

## Fix projects and dependencies

If you are working on a not so small Fix program, you may want to
- compile many Fix source files,
- compile C source files into a native library, and link it to the Fix program,
- install other Fix projects as dependencies, 
- specify the project name, version or author, etc.
In such cases, it is useful to have a project file which contains information about your Fix project.

The project file should have a name "fixproj.toml".
"fix build" and "fix run" commands loads the project file in the current directory if it exists.

The following is an example of "fixproj.toml" file. 

```
[general]
# Project name. This is a required field.
name = "myproject"

# Project version (in semver). This is a required field.
version = "0.1.0"

# Project authors.
authors = ["Alice", "Bob"]

# Project description.
description = "This is a Fix project."

# Project license.
license = "MIT"

[build]
# Fix source files to be compiled.
# Merged with files specified in the command line argument.
files = ["main.fix", "lib.fix"]

# Static link libraries.
# Merged with libraries specified in the command line argument.
static_links = ["xyz"]

# Dynamic link libraries.
# Merged with libraries specified in the command line argument.
dynamic_links = ["pthread"]

# Library search paths for "static_links" and "dynamic_links".
# Merged with paths specified in the command line argument.
library_paths = ["."] // The current directory.

# Whether to generate debug information.
# Overwritten by the command line argument.
debug = true

# Optimization level.
# One of "none", "minimum", "separated", "default".
# Overwritten by the command line argument.
opt_level = "separated"

# Output file name.
# Overwritten by the command line argument.
output = "myprogram.out"

# Whether to use the thread-safe reference counting.
# Overwritten by the command line argument.
threaded = false

# Preliminary commands to be executed before the Fix program is compiled.
# This is useful when you need to compile a C library before compiling the Fix program.
preliminary_commands = [
    ["make", "libxyz.a", "--quiet"] # Since this command runs always when you run "fix build", "--quiet" is useful.
]

# By "[[dependencies]]" array, you can specify a Fix project as a dependency.
# The dependent project should have "fixproj.toml" file, which at least defines name and version of the project.
# If a dependent project has more dependencies, fix command will consider them recursively.

# The following is an example of a dependency to a project in the local file system.
[[dependencies]]
name = "another-project"
version = "*"
path = "/path/to/project"

# The following is an example of a dependency to a project published in the GitHub.
[[dependencies]]
name = "certain-project"
version = "1.2.0"
git = { url = "https://github.com/tttmmmyyyy/certain-project.git" }
```

Even if you specify source files in the project file, you can specify additional source files in the command line argument by "-f".
For example, consider that you are making a library and having `Main::main` function which runs tests in "test.fix".
In this case, you should not include "test.fix" in the "build.files" of the project file, because otherwise the test codes will be compiled into the application that uses your library.
Instead, you can add "test.fix" by the command line argument "-f" when you run the test.

## Language Server Protocol

Running `fix language-server` starts a language server which supports Language Server Protocol (LSP). 
Language client extension for VSCode is available in [here](https://marketplace.visualstudio.com/items?itemName=tttmmmyyyy.fixlang-language-client).

The language server requires [the project file](#project-file) to recognize the Fix source files.

Each time you save a file, the language server will attempt to diagnose the Fix program.
The information obtained in the latest successful diagnostics is used to comletion, hover or go-to-definition, etc.
So to update the information, you need to write correct Fix code and save the file. 
[`Std::undefined`](./BuiltinLibraries.md#undefined--lazy-a) will be useful to do so.

## Debugging

Running `fix build` with `-g` option generates executable binary with DWARF debugging information. Then you can debug the binary by lldb, gdb or other GUI debuggers such as [CodeLLDB](https://marketplace.visualstudio.com/items?itemName=vadimcn.vscode-lldb). 

In VSCode, you cannot put a breakpoint in *.fix files by default. As a workaround, open "Preferences" and turn "Allow Breakpoints Everywhere" ON.

There are some notes on debugging Fix program:
- Unlike other languages, Fix does not release local variables at the end of their scope, but at the last point of use. So if you break after the last use of a local variable, the debugger may show you an invalid value.
- Currently, we are not able to tell the debugger the size of an array which is determined at run time. So we are always setting the array size to 100 in the debug information. You cannot show elements indexed after 100, and if the array is shorter than 100, invalid values are shown.