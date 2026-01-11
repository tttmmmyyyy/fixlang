# Table of contents

- [Table of contents](#table-of-contents)
- [Tutorial](#tutorial)
    - [Set up the tools](#set-up-the-tools)
        - [Fix compiler](#fix-compiler)
            - [Use pre-built binary](#use-pre-built-binary)
            - [Build from source](#build-from-source)
            - [Use Docker image](#use-docker-image)
        - [(Optional) VScode extensions](#optional-vscode-extensions)
    - [Running Your First Fix Program](#running-your-first-fix-program)
    - [Modules](#modules)
    - [Global values](#global-values)
        - [Namespaces](#namespaces)
    - [Types](#types)
    - [Expressions](#expressions)
    - [Let-expressions](#let-expressions)
    - [If-expressions](#if-expressions)
    - [Function application](#function-application)
    - [Function definition](#function-definition)
    - [The `.` and `$` Operators](#the--and--operators)
    - [Patterns](#patterns)
    - [The `loop`, `continue`, and `break` Functions](#the-loop-continue-and-break-functions)
    - [Unions](#unions)
    - [Structs](#structs)
    - [Iterators](#iterators)
    - [Mutability and Reference Counting in Fix](#mutability-and-reference-counting-in-fix)
    - [A bit on IO (or monads)](#a-bit-on-io-or-monads)
- [More on language and standard library](#more-on-language-and-standard-library)
    - [Boxed and Unboxed Types](#boxed-and-unboxed-types)
        - [Functions](#functions)
        - [Tuples and unit](#tuples-and-unit)
        - [Array](#array)
        - [Structs](#structs-1)
        - [Unions](#unions-1)
    - [Booleans and literals](#booleans-and-literals)
    - [Numbers and literals](#numbers-and-literals)
    - [Strings and literals](#strings-and-literals)
    - [Arrays and literals](#arrays-and-literals)
    - [Unit and tuples](#unit-and-tuples)
    - [Structs](#structs-2)
        - [`@f : S -> F`](#f--s---f)
        - [`set_f : F -> S -> S`](#set_f--f---s---s)
        - [`mod_f : (F -> F) -> S -> S`](#mod_f--f---f---s---s)
        - [`act_f : [f : Functor] (F -> f F) -> S -> f S`](#act_f--f--functor-f---f-f---s---f-s)
    - [Unions](#unions-2)
        - [`v : V -> U`](#v--v---u)
        - [`is_v : U -> Bool`](#is_v--u---bool)
        - [`as_v : U -> V`](#as_v--u---v)
        - [`mod_v : (V -> V) -> U -> U`](#mod_v--v---v---u---u)
    - [Index Syntax](#index-syntax)
    - [Modules and import statements](#modules-and-import-statements)
    - [Namespaces and overloading](#namespaces-and-overloading)
    - [More on import statements: filtering entities](#more-on-import-statements-filtering-entities)
    - [Absolute Namespace and Imports](#absolute-namespace-and-imports)
    - [Which is better: importing whole module or only necessary entities?](#which-is-better-importing-whole-module-or-only-necessary-entities)
    - [Recursion](#recursion)
    - [Type annotation](#type-annotation)
    - [Pattern matching](#pattern-matching)
    - [Traits](#traits)
    - [Associated types](#associated-types)
    - [Trait alias](#trait-alias)
    - [Type alias](#type-alias)
        - [Dynamic Iterators](#dynamic-iterators)
    - [Monads](#monads)
        - [What is monad?](#what-is-monad)
        - [Stateful Monads](#stateful-monads)
            - [Failure Monads](#failure-monads)
        - [Sequence Monads](#sequence-monads)
        - [`do` Blocks and the monadic bind operator `*`](#do-blocks-and-the-monadic-bind-operator-)
        - [When an explicit `do` block is needed](#when-an-explicit-do-block-is-needed)
        - [Chaining monadic actions with the `;;` Syntax](#chaining-monadic-actions-with-the--syntax)
        - [Fix's Iterator is not a monad](#fixs-iterator-is-not-a-monad)
    - [Foreign Function Interface (FFI)](#foreign-function-interface-ffi)
        - [Calling External Functions from Fix](#calling-external-functions-from-fix)
        - [Exporting Fix Values and Functions to External Languages](#exporting-fix-values-and-functions-to-external-languages)
        - [Managing External Resources in Fix](#managing-external-resources-in-fix)
        - [Managing ownership of Fix's boxed value in a foreign language](#managing-ownership-of-fixs-boxed-value-in-a-foreign-language)
        - [Accessing fields of Fix's struct value from C](#accessing-fields-of-fixs-struct-value-from-c)
    - [`eval` syntax](#eval-syntax)
    - [Substitute Pattern](#substitute-pattern)
    - [Operator and Syntax Precedence](#operator-and-syntax-precedence)
- [Compiler features](#compiler-features)
    - [Project file](#project-file)
        - [About Duplicated Build Settings](#about-duplicated-build-settings)
    - [Managing dependencies](#managing-dependencies)
    - [Configuration file](#configuration-file)
    - [Generating documentation](#generating-documentation)
    - [Language Server Protocol](#language-server-protocol)
        - [Specifying parameter list in the documentation comment as a hint to the language server](#specifying-parameter-list-in-the-documentation-comment-as-a-hint-to-the-language-server)
    - [Debugging Fix program](#debugging-fix-program)
- [Other documents](#other-documents)

# Tutorial

## Set up the tools

### Fix compiler

Currently, Fix compiler is supported on macOS / Linux / Windows (via WSL). You can prepare the compiler one of the following ways.

#### Use pre-built binary

You can download pre-built compiler binary from [Releases](https://github.com/tttmmmyyyy/fixlang/releases/).
Download it, rename it to "fix", and place it to "/usr/local/bin" or somewhere else.

#### Build from source

Fix compiler is written in Rust. Thanks to Cargo, it is easy to build the compiler from source.

1. Install [Rust](https://www.rust-lang.org/tools/install).
2. Install LLVM 17.0.x.
- In Linux / WSL, you can download prebuilt binary of LLVM from [LLVM Download Page](https://releases.llvm.org/download.html).
- In macOS, you can get LLVM by `brew install llvm@17`.
3. Set `LLVM_SYS_170_PREFIX` variable to the directory to which LLVM is installed.
- If you installed LLVM by `brew`, you can set it by `export LLVM_SYS_170_PREFIX=$(brew --prefix llvm@17)`. 
4. `git clone https://github.com/tttmmmyyyy/fixlang.git && cd fixlang`.
5. `cargo install --locked --path .`. Then the command `fix` will be installed to `~/.cargo/bin`.

#### Use Docker image

Thanks to [pt9999](https://github.com/pt9999), [docker image](https://hub.docker.com/r/pt9999/fixlang) is available! 

### (Optional) VScode extensions

If you are using VScode, we recommend you to install the following extensions:

- [Syntax highlighting](https://marketplace.visualstudio.com/items?itemName=tttmmmyyyy.fixlangsyntax)
- [Language client](https://marketplace.visualstudio.com/items?itemName=tttmmmyyyy.fixlang-language-client)

## Running Your First Fix Program

Below is a Fix program that calculates the first 30 numbers of the Fibonacci sequence.

```fix
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
    println("The first 30 numbers of Fibonacci sequence are: ");;
    println $ fib.to_iter.map(to_string).join(", ")
);
```

To run this program, first create a working directory for your Fix project. In that directory, run `fix init` to create the Fix project templates (`"fixproj.toml"`, `"main.fix"`, and `"test.fix"`). Next, copy the source code above into the `"main.fix"` file.

The project file `"fixproj.toml"` tells the Fix compiler about your project's configuration and how to build it. The default project file created by `fix init` includes the following, so the `"main.fix"` source file is recognized as a build target:

```toml
[build]
files = ["main.fix"]
```

Running `fix run` in your working directory will compile and execute the program. The following output should be displayed on standard output:

```
The first 30 numbers of Fibonacci sequence are: 
1, 1, 2, 3, 5, 8, 13, 21, 34, 55, 89, 144, 233, 377, 610, 987, 1597, 2584, 4181, 6765, 10946, 17711, 28657, 46368, 75025, 121393, 196418, 317811, 514229, 832040
```

Alternatively, you can run `fix build` to have the compiler generate an executable binary (`"a.out"`), which you can then run with `./a.out`.

This is the basic usage of the Fix compiler. For more details on the compiler's features, refer to the [Compiler Features](https://www.google.com/search?q=%23compiler-features) section.

Below, we'll explain the syntax and meaning of the sample program above.

## Modules

The first line of "main.fix" is a module definition.

```
module Main;
```

In Fix, values, functions, types, and traits defined in a source file are grouped into a single **module**. Each source file must specify the name of the module it defines using `module {module_name};`.

When a Fix program is executed, the `main` function defined in the `Main` module is called.

Module names must begin with a capital letter. Additionally, you can use a string of these names concatenated with a period (e.g., `Main.Model.Impl`) as a module name, which is useful for representing a hierarchical module structure.

## Global values

The following parts are definitions of two global values `calc_fib` and `main`.

```fix
calc_fib : I64 -> Array I64;
calc_fib = {expression A};

main : IO ();
main = {expression B};
```

These lines means that:

- `calc_fib` global value has type `I64 -> Array I64` and its value is defined by `{expression A}`.
- `main` global value has type `IO ()` and its value is defined by `{expression B}`.

In Fix, you have to specify the type of a global value explicitly. 

NOTE: Since version 1.1.0 of Fix, the above can be written more concisely as follows.

```fix
calc_fib : I64 -> Array I64 = {expression A};
```

### Namespaces

In `Array::fill`, `Array` is a namespace. A namespace is like an address for a name, used to distinguish between two values (or types, traits, or anything defined globally) that have the same name.

In many cases, the namespace can be omitted. In fact, with the current version of the standard library, you can write `fill(n, 0)` instead of `Array::fill(n, 0)`. This is because the compiler can infer from the context that the value written as `fill` refers to `Array::fill`.

Actually, the "full name" of `fill` is `Std::Array::fill`, not just `Array::fill`. `Std` is the module for the standard library. Modules are used as top-level namespaces. `Std::Array::fill` means that the function `fill` is defined in the namespace `Array`, which is inside the module `Std`.

Even though you can simply write `fill(n, 0)`, the sample program uses `Array::fill(n, 0)` for the following reasons:

- `Array::fill(n, 0)` is considered more readable than `fill(n, 0)` because it expresses that this `fill` function creates an `Array` type.
- In the future, a function named `fill` might be added to a different namespace besides `Array`. In this case, the name `fill` would become ambiguous, and the sample program might fail to compile.

Similarly, the "full name" of the `calc_fib` function is `Main::calc_fib`.

## Types

Each value in Fix has a type. Using the terminology of mathematics, a type can be considered as a set, and a value in Fix is an element of that set.

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
- `I64 -> Bool -> Array Bool`: this is equivalent to `I64 -> (Bool -> Array Bool)`, that is, the type of functions that receives an integer and returns a function that converts a boolean value into a boolean array. As an example, a function that produces a boolean array from its length and initial value has this type. In Fix, there is no concept of "two-variable functions". The type of something like "two-variable functions" can be represented as `a -> b -> c` or `(a, b) -> c`.

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

This `in` and `;` are synonymous. Use the one you prefer.

If you want to put `{epxression_0}` and `{expression_1}` in other lines, it is better to use semicolon:

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

which is evaluated to `128`, can also be written as 

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

and the program

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

has the same meaning, but the former is more readable and recommended.

## The `.` and `$` Operators

The `.` operator is another way to apply a function to a value. It is defined as `x.f == f(x)`.

The precedence of the `.` operator is lower than that of function application using parentheses. Therefore, if a function `method` has the type `Param -> Obj -> Result`, `obj.method(arg)` is interpreted as `obj.(method(arg)) == method(arg)(obj) == method(arg, obj)`, not `(obj.method)(arg)`.

In the Fibonacci program, here are examples of the `.` operator's usage:

* `arr.get_size`: `get_size` is a function of type `Array a -> I64` that returns the length of an array. You should not write `arr.get_size()` like in other languages. Simply writing `arr.get_size` has the same meaning as `get_size(arr)`.
* `arr.set(0, 1)`: `set` is a function of type `I64 -> a -> Array a -> Array a` that updates an array's element with a specified value.
* `arr.@(idx-1)`: `@` is a function of type `I64 -> Array a -> a` that returns the element at a specified index.

A function of type `Param0 -> ... -> ParamN -> Obj -> Result` is sometimes called a "method" of type `Obj` that takes N+1 parameters and returns a value of type `Result`. A method can be called by writing `obj.method(arg0, ..., argN)` like in OOP languages.

Another way to apply a function is the `$` operator: `f $x = f(x)`. This operator is right-associative: `f$ g $ x = f(g(x))`.

The `$` operator is useful for reducing parentheses. In the Fibonacci program, here are examples of its usage:

* `continue $ (idx+1, arr)`: This applies the `continue` function to the tuple value `(idx+1, arr)`. In Fix, `continue` and `break` are regular functions, not keywords. Therefore, this expression could also be written as `continue((idx+1, arr))` or `(idx+1, arr).continue`. A detailed explanation of the `continue` and `break` functions is provided later.
* `println $ fib.to_iter.map(to_string).join(", ")`: This applies the `println` function to the string expression `fib.to_iter.map(to_string).join(", ")`. Since the `println` function has the type `String -> IO ()`, applying it to a string produces a value of type `IO ()`. This expression could also be written as `println(fib.to_iter.map(to_string).join(", "))`, but using the `$` operator can reduce parentheses around long string expressions.

The precedence of the three function application methods is `f(x)` > `x.f` > `f $x`. For this reason, you cannot write `obj.method$ arg`. This would be equivalent to `method(obj) $arg == method(obj, arg)`, which tries to call the method with two arguments in the wrong order. On the other hand, you can write `method(arg)$ obj`, which reads as "apply `method` to `arg` to get a function of type `Obj -> Result`, and then apply that to `obj`."

## Patterns

In Fix, you can use pattern matching for structs and tuples in `let` expressions, `match` expressions, and function expressions.

For example, let's define a function `swap` that takes a value of the tuple type `(I64, Bool)` and returns a value of type `(Bool, I64)`. Without using patterns, you could write it like this, using the built-in functions `@0 : (a, b) -> a` and `@1 : (a, b) -> b` to extract elements from the tuple:

```
swap : (I64, Bool) -> (Bool, I64);
swap = |tuple| (
    let fst = tuple.@0;
    let snd = tuple.@1;
    (snd, fst)
);
```

By using a pattern in a `let` expression, the program can be written as follows:

```
swap : (I64, Bool) -> (Bool, I64);
swap = |tuple| (
    let (fst, snd) = tuple;
    (snd, fst)
);
```

Alternatively, you can use a pattern in the function expression to write it like this:

```
swap : (I64, Bool) -> (Bool, I64);
swap = |(fst, snd)| (snd, fst);
```

Note: Do not confuse `|(x, y)| ...` with `|x, y| ...`. The former defines a function that accepts a single tuple, while the latter defines a function that accepts two separate arguments.

## The `loop`, `continue`, and `break` Functions

The built-in `loop` function is used to implement loops in Fix. To continue or break a loop, you use the `continue` and `break` functions.

The types of `loop`, `continue`, and `break` are as follows:

  - `loop : s -> (s -> LoopState s b) -> b`
  - `continue : s -> LoopState s b`
  - `break : b -> LoopState s b`

The `loop` function takes two arguments: an initial state `s0` for the loop and a loop body function `body`. The `loop` function first calls `body` with `s0`. If `body` returns a `break(r)` value, the `loop` function terminates and returns `r` as the result. If `body` returns a `continue(s)` value, the `loop` function calls `body` again with `s`.

In the Fibonacci program, the `loop` function is used in the following expression:

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

The initial state of this loop is `(2, arr)`. The loop body accepts a state of tuple type `(idx, arr)`. Here, `idx` is the index of the array to be updated next, and `arr` is the array of Fibonacci numbers where indices from `0` to `idx-1` have already been computed.

If `idx` reaches `arr.get_size`, the loop terminates by returning `break $arr`. Otherwise, it computes the Fibonacci number at index `idx`, stores it in `arr`, and then returns `continue$ (idx+1, arr)` to continue the loop.

## Unions

Then what is the type `LoopState s b`? It is defined as an union with two type parameters `s` and `b`. It can be defined as follows:

```
type LoopState s b = union { continue : s, break : b };
```

The above definition indicates that a `LoopState s b` value contains either of a value of type `s` or a value of type `b`. If you write the set of values of a type as `|type|`, then `|LoopState s b| = |s| ⨆ |b|`, where the symbol `⨆` is represents the disjoint union of sets.

For each union type, some basic methods are automatically defined. For example, for `LoopState` as above, the following functions are defined in the namespace `LoopState`.

- `continue : s -> LoopState s b`: converts an value of type `s` into a `LoopState` value.
- `break : b -> LoopState s b`: converts an value of type `b` into a `LoopState` value.
- `is_continue : LoopState s b -> Bool`: checks if the `LoopState` value was created by `continue`.
- `is_break : LoopState s b -> Bool`: checks if the `LoopState` value was created by `break`.
- `as_continue : LoopState s b -> s`: extracts a value of type `s` from a `LoopState` value if it is created by `continue`. If not, this function aborts the program.
- `as_break : LoopState s b -> s`: extracts a value of type `b` from a `LoopState` value if it is created by `break`. If not, this function aborts the program.

Another example of union is `Option` which is used to represent a value "which may not contain a value". It is defined as follows: 

```
type Option a = union { none : (), some : a };
```

Note that, if you want to create a none value of `Option`, you need to write `none()`, because `none` is a function of type `() -> Option a`. (Remember that the syntax sugar `f() == f(())`.)

## Structs

Although it does not appear in the example Fibonacci program, here I explain how to define your own struct.

For example, you can define a struct called `Product` with two fields `price`  of type `I64` and `sold` of type `Bool` as follows.

```
type Product = struct { price: I64, sold: Bool };
```

You can construct a struct value by the syntax `{struct_name} { {field_name}: {field_value} } `:

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

Now I explain about the expression `fib.to_iter.map(to_string).join(", ")`, where `fib : Array I64` is the array of Fibonacci sequence. This expression 
- converts a Fibonacci array into an iterator of integers, 
- apply `to_string : I64 -> String` to each element to obtain the iterator of strings, and
- concatenates these strings separated by `", "`,
- results in a string "1, 1, 2, 3, 5, 8, 13, 21, 34, 55, 89, 144, 233, 377, 610, 987, 1597, 2584, 4181, 6765, 10946, 17711, 28657, 46368, 75025, 121393, 196418, 317811, 514229, 832040".

An iterator is a concept of a sequence of elements that can be iterated similar to arrays or singly linked lists.
More precisely, an iterator is a type whose data is "the current state" and has a method `advance` which returns the next element and the next state.

Since it does not store all data in memory at once like arrays or singly linked lists, iterators can use memory efficiently.
Also, it can represent an infinite sequence of data.

In Fix, an iterator is defined as the following trait.

```
// The trait of iterators.
// 
// Iterator is a concept of a sequence of elements that can be iterated.
// More precisely, an iterator is a type whose data is "the current state" and has a method `advance` which returns the next element and the next state.
trait iter : Iterator {
    type Item iter;
    advance : iter -> Option (iter, Item iter);
}
```

A trait is a concept that represents the properties that a type should satisfy.
The above definition indicates that a type `iter` should have a type `Item iter` and a state transition function `advance` to be an `Iterator`.

Let's see a simple example of an iterator.
An iterator that counts up from a number is created by the following function.

```
// Create an iterator that counts up from a number.
// 
// `count_up(start)` generates an infinite sequence of numbers starting from `start`.
count_up : I64 -> CountUpIterator;
count_up = |start| CountUpIterator { next: start };
```

Here is the definition of `CountUpIterator` and the implementation of `Iterator` trait.

```
type CountUpIterator = unbox struct { next : I64 };

impl CountUpIterator : Iterator {
    type Item CountUpIterator = I64;
    advance = |CountUpIterator { next : next }| some((CountUpIterator { next: next + 1 }, next));
}
```

In the expression `fib.to_iter.map(to_string).join(", ")` of the example program, first, the array is converted to an iterator by the `to_iter` function.
The type of the `to_iter` function is as follows.

```
// Converts an array to an iterator.
to_iter : Array a -> ArrayIterator a;
```

`ArrayIterator` is a type that holds an array and the current index as data, and an implementation of `ArrayIterator a : Iterator` is given in the standard library.

`map` is a function that applies a function to each element of an iterator and generates a new iterator.

```
// Map a function over an iterator.
// 
// `iter.map(f)` returns an iterator that applies `f` to each element of `iter`.
map : [i : Iterator, Item i = a] (a -> b) -> i -> MapIterator i a b;
```

`to_string` is a function that converts an integer to a string, and by `map(to_string)`, the iterator of integers is converted to the iterator of strings.

`join` is a function that takes an iterator of strings and a separator, and joins the strings.

```
// Joins (an iterator of) strings by a separator.
join : [ss : Iterator, Item ss = String] String -> ss -> String;
```

I hope you can understand the behavior of `fib.to_iter.map(to_string).join(", ")` now.

In the example program, I introduced the `loop` function to realize loops, but sometimes it is more concise to create an iterator of the range to loop over and use a function to loop along the iterator.

A representative example of a function that loops along an iterator is `fold`.

```
// Fold the elements of an iterator from left to right.
//
// Conceptually, `[a0, a1, a2, ...].fold(s, op) = s.op(a0).op(a1).op(a2)...`.
fold : [iter : Iterator, Item iter = a] s -> (a -> s -> s) -> iter -> s;
```

The `fold` function creates state update functions `op(a0)`, `op(a1)`, ... from the elements of the iterator, and applies these state update functions in order to calculate the final state.

By using `fold`, the `calc_fib` function in the example program can be written as follows.

```
calc_fib : I64 -> Array I64;
calc_fib = |n| (
    let arr = Array::fill(n, 0);
    let arr = arr.set(0, 1);
    let arr = arr.set(1, 1);
    let arr = Iterator::range(2, n).fold(arr, |idx, arr|
        let x = arr.@(idx-1);
        let y = arr.@(idx-2);
        arr.set(idx, x+y)
    );
    arr
);
```

Note that `fold` cannot break in the middle of the loop. If you need to break in the middle, use the `loop_iter` function.

```
// Loop over the elements of an iterator.
// 
// This function is similar to `fold` but a more general version of it. It allows the user to break out of the loop at any point.
loop_iter : [iter : Iterator, Item iter = a] s -> (a -> s -> LoopState s s) -> iter -> s;
```

## Mutability and Reference Counting in Fix

Remember that Fix expressions are just strings describing values. They are essentially the same as a mathematical expression like "1 + cos(pi/5)^2". The concept of "changing a variable's value," which is widely used in conventional languages, does not exist. All values in Fix are **immutable**.

For example, consider the following code:

```
main = (
    let arr0 = Array::fill(100, 1);
    let arr1 = arr0.set(0, 2);
    println("arr0.@(0): " + arr0.@(0).to_string + ".")
);
```

The code above will print `arr0.@(0): 1.`, not `2`. This is because `arr0.set(0, 2)` is an expression that represents "another array with the 0th element of `arr0` changed to `2`," not a command to "update the 0th element of `arr0` to `2`."

To achieve this behavior, the `set` function in the program above must create a copy of `arr0`, update its 0th element to `2`, and then return the new array.

Now, let's consider the implementation of `calc_fib`:

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

The optimal time complexity for calculating an N-length Fibonacci sequence is O(N). However, if Fix were to copy the array in `let arr = arr.set(idx, x+y);` within the loop, each loop step would take O(N) time, and the total time complexity would become O(N^2).

In reality, the `set` in the program above does not copy the array, and `calc_fib` runs in the expected O(N) time. This is because `set` performs an optimization: it skips the copy and modifies the given array in place, but only if the given array will not be used again.

Consider the following program:

```
main = (
    let arr0 = Array::fill(100, 1);
    let arr1 = arr0.set(0, 2);
    println("arr1.@(0): " + arr1.@(0).to_string + ".")
);
```

(Note that `println` prints the 0th element of `arr1`, not `arr0`.) In this program, the call to `set` is the last use of `arr0`. In such a case, `set` updates the given array in place without copying it. This does not compromise Fix's immutability because the modification of `arr0` is never observed.

Let's return to the `calc_fib` function. In the line `let arr = arr.set(idx, x+y);`, the name `arr` is redefined and set to point to the new array returned by the `set` function. This ensures that the old array passed to the `set` function is never referenced after this line. Therefore, it is clear that the `set` function does not need to copy the given array, and in practice, no copy is made.

To summarize:

- Since Fix values are immutable, the `set : I64 -> a -> Array a -> Array a` function can fundamentally be interpreted as returning a new array.
- However, if the array is not used later, the copy is omitted, and the given array is updated in place.

Fix determines whether a value may be used later by its **reference count**. Fix assigns a reference counter to all boxed values (values that are always allocated on the heap and referenced by a pointer from a name or a struct field). Fix uses the reference counter to track the number of references to a boxed value. When the reference counter is 1, the value is called "unique"; otherwise, it is called "shared." For convenience, an unboxed value is always considered unique.

Using these terms, the `set` function modifies the array directly if it is unique but copies it before modifying it if it is shared.

When implementing algorithms like dynamic programming that depend on modifying an array in O(1) time, it is critical to pass a unique array to `set`. So, how can you guarantee that the array passed to `set` is unique? As we saw, when `arr.set(idx, v)` is the last use of `arr`, `arr` is unique at the call to `set` (\*). Specifically, by writing `let arr = arr.set(idx, v);`, you guarantee that `set` receives a unique array, because the new, updated array shadows the old array's name, so the old array is never used after the call to `set`.

(\*): An exception is when `arr` is referenced by multiple threads.

## A bit on IO (or monads)

Let's see the last few lines of the sample code.

```
main : IO ();
main = (
    let fib = calc_fib(30);
    println("The first 30 numbers of Fibonacci sequence are: ");;
    println $ fib.to_iter.map(to_string).join(", ")
);
```

`println : String -> IO ()` is a function that takes a string and produces an IO action that prints the string to the screen. 
In this code, two IO actions created by two `println` are combined by double-semicolon syntax (`;;`) to create a larger IO action that prints two lines to the screen.

How to combine IO actions and more generally, how to combine monads to create more complex monads are explained in [Monads](#monads).

# More on language and standard library

## Boxed and Unboxed Types

Fix types are divided into **boxed** and **unboxed** types, which are similar to what other languages call "reference types" and "value types."

* **Boxed** type values are allocated on the heap. A local variable or a field in a struct/union with a boxed type is compiled as a pointer to the value.
* **Unboxed** type values are directly embedded in stack memory, structs, or unions.

In general, it's recommended that types containing a large amount of data (e.g., `Array a`) be **boxed** to reduce copying costs. On the other hand, types with little data (e.g., `I64`) can be **unboxed** to eliminate the overhead of incrementing and decrementing reference counters and to improve memory locality.

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

## Booleans and literals

The type for booleans is `Bool`, and literals for booleans are `true` and `false`.

## Numbers and literals

Types for numbers are `I8`, `I16`, `I32`, `I64` (signed integers), `U8`, `U16`, `U32`, `U64` (unsigned integers) and `F32`, `F64` (floating point values).

A number literal is interpreted as a floating point literal if it contains a decimal point, and as an integer literal otherwise.
For example, `42` is an `I64` type number literal, and `3.14` is an `F64` type number literal.

To write a number literal of a type other than `I64` and `F64`, write an underscore and the type name after the literal.
For example, `42_I32` is an `I32` type number literal, and `3.14_F32` is an `F32` type number literal.

Integer literals are represented in decimal by default, and can be represented in hexadecimal with a `0x` prefix, in octal with a `0o` prefix, and in binary with a `0b` prefix.
For example, `0x2A` represents 42, and `0o52` also represents 42.

In integer literals in decimal, you can use "e" to represent the power of 10.
For example, `4e2` represents 400.

Characters enclosed in single quotes are interpreted as `U8` type number literals.
For example, `'A'` represents 65.

Additionally, `\n`, `\r`, `\t`, `\0`, `\\`, `\'` are interpreted as `U8` type number literals representing the character codes of newline, carriage return, tab, null character, backslash, and single quote, respectively.

Note that floating point literals must have at least one digit before and after the decimal point.
For example, `1.` and `.1` are not valid floating point literals (while they are valid in C).

## Strings and literals

`String` is a type representing a string. Internally, it is represented as a null-terminated array of `U8`.

A string literal is a string enclosed in double quotes.
For example, `"Hello, world!"` is a string literal of type `String`.

In a string literal, `\n`, `\r`, `\t`, `\\`, `\"` are interpreted as newline, carriage return, tab, backslash, and double quote, respectively.

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

## Index Syntax

Consider the following struct:
```
type Vector = struct { x: F64, y: F64 };
```

In this situation, suppose we define an array `vs` as follows:
```
let vs = [Vector { x: 1.0, y: 2.0 }, Vector { x: 3.0, y: 4.0 }];
```

If we want to change the value of the `x` field of the first element of `vs` to `5.0`, we can use the index syntax to write:

```
let vs = vs[0][^x].iset(5.0);
```

Here, `[0]` means accessing the 0th index of the array, and `[^x]` means accessing the `x` field.
The expression `vs[0][^x]` using index syntax generates a value called a "store" (of type `[f : Functor] (F64 -> f F64) -> f (Array Vector)`) to the region we want to access, and `iset` works to set a value to the store.

Without using index syntax, you can also write:
```
let vs = vs.mod(0, set_x(5.0));
```
The deeper the hierarchical structure of types becomes, the more concisely you can write using index syntax.

In addition to `iset`, there are other functions that operate on stores: `iget`, `imod`, and `iact`.

`iget` retrieves a value from the store.
```
let vs = [Vector { x: 1.0, y: 2.0 }, Vector { x: 3.0, y: 4.0 }];
let v0x = vs[0][^x].iget; // v0x == 1.0
```

`imod` modifies the entire value through a function acting on the store.
```
let vs = [Vector { x: 1.0, y: 2.0 }, Vector { x: 3.0, y: 4.0 }];
let vs = vs[0][^x].imod(|x| x + 10.0); // vs == [Vector { x: 11.0, y: 2.0 }, Vector { x: 3.0, y: 4.0 }]
```

`iact` is similar to `imod`, but modifies the entire value through a functorial action.
```
let f = |x| println(x.to_string + "!");; (-x).pure; // An IO action that prints `x` to the screen and returns its negative value
let vs = [Vector { x: 1.0, y: 2.0 }, Vector { x: 3.0, y: 4.0 }];
let vs = *vs[0][^x].iact(f); // Prints `1.0!` and results in `vs == [Vector { x: -1.0, y: 2.0 }, Vector { x: 3.0, y: 4.0 }]`
```

When specifying a field using index syntax (`obj[^field]`), if the field name is ambiguous and causes a compilation error, you can specify the namespace of the field like `obj[^NameSpace::field]`.

Index syntax is syntactic sugar. `vs[0][^x]` is expanded to `|f| vs.(act(0) << act_x)(f)`.

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
import Std::{IO, String, IO::println};

main : IO ();
main = println("Hello, World!");
```

If you want to import `Std::IO::eprintln` in addition, you can write:

```
import Std::{IO, String, IO::println, IO::eprintln};
```

or

```
import Std::{IO, String, IO::{println, eprintln}};
```

If importing any entities in the `Std::IO` namespace is OK, you can write:

```
module Main;
import Std::{IO, String, IO::*};

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

## Absolute Namespace and Imports

Suppose you define `Main::truth` in the `Main` module.
Even if you fully specify the namespace as `Main::truth`, there may be cases where overload resolution fails.
This happens in situations like the following:

```
module Main;

truth : I64 = 42; // Main::truth

namespace Main {
    truth : I64 = 42; // Main::Main::truth
}

main : IO ();
main = Main::truth.to_string.println; // error: Name `Main::truth` is ambiguous: there are `Main::Main::truth`, `Main::truth`.
```

In this program, there are two `truth` values: `Main::truth` and `Main::Main::truth`, so just writing `Main::truth` doesn't clarify which `truth` you're referring to.

In such cases, you can tell Fix that you're writing a "fully qualified name" by adding `::` at the beginning of the namespace, like `::Main::truth`.

```
module Main;

truth : I64 = 42; // Main::truth

namespace Main {
    truth : I64 = 42; // Main::Main::truth
}

main : IO ();
main = ::Main::truth.to_string.println; // OK
```

Additionally, when you reference an entity using such an "absolute namespace", that entity can be used even if it hasn't been imported.
For example, the following program doesn't import anything from `Std`, but uses `IO`, `pure`, etc. by specifying their absolute namespace.

```
module Main;

import Std::{}; // Hide all standard library entities

main : ::Std::IO ();
main = ::Std::Monad::pure();
```

In Fix, import statements are syntax for adding or removing candidates in overload resolution.
When you reference an entity by specifying its absolute namespace, overload resolution is not needed, so import statements are ignored.

## Which is better: importing whole module or only necessary entities?

Which is better: importing an entire module like this:

```
import Lib;
```

or importing only the necessary entities like this:

```
import Lib::{value0, Type1};
```

The latter requires you to update the `import` statement every time you use an entity from `Lib`.
The former avoids this overhead.
If you want to code speedily, adopting this style is recommended.

On the other hand, the latter has advantages from a maintenance perspective.
Suppose your code defines a value named `value`.
Then, the library `Lib` is updated and adds a value with the same name `value`.
In this case, if you had imported the entire `Lib` module, `value` would become ambiguous in your code, potentially resulting in a compilation error.
If you only import the necessary entities, you can avoid this problem.

In Fix, to make it easy to start writing programs, `import Std;` is implicitly performed.
Therefore, when functionality is added to the `Std` module, there is a possibility that the names of entities defined in your code will collide with the names of newly added entities in the `Std` module, resulting in a compilation error.
If you want to avoid this, we recommend a style where you write `import Std::{};` at the beginning and import the necessary entities (entities that cause "Unknown name" errors) as needed.

When adopting a style of importing only the necessary entities, the following tools are useful:
- The Language Server Protocol support of the Fix compiler can automatically update `import` statements through entity name completion operations or Quick Fix for "Unknown name" errors.
- Using the `fix edit explicit-import` command, you can automatically rewrite `import` statements to import only the necessary entities based on the entities used in the source file.

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

## Type annotation

You need to write types of global value explicity. You can specify the type of a local value for readability or for helping type / namespace inference of Fix compiler.

The following demonstrates type annotations for local values.

```
module Main;

main : IO ();
main = (
    let x = 42 : I64; // Type annotation on expression.
    let y : I64 = 42; // Type annotation on let-binding.
    let f = |v : I64| v * 3; // Type annotation on the parameter of a function.
    
    println $ x.to_string;;
    println $ y.to_string;;
    println $ f(14).to_string;;

    pure()
);
```

## Pattern matching

Pattern matching is a syntax for extracting values from structs (including tuples) or unions.
Pattern matching for structs can be used in function arguments or let-bindings. 
Pattern matching for unions can be used in `match` expressions.

Examples:
```
module Main;

type IntBool = struct { i : I64, b : Bool };

to_pair : IntBool -> (I64, Bool);
to_pair = |IntBool { i : x, b : y }| (x, y); // Pattern matching for function argument

main : IO ();
main = (
    let int_bool = IntBool { i : 42, b : true };
    let (i, b) = to_pair(int_bool); // Pattern matching at let-binding
    println $ "(" + i.to_string + ", " + b.to_string + ")"
);
```

```
module Main;

main : IO ();
main = (
    let opt = Option::some(42);

    let x = match opt {
        some(v) => v,
        none(_) => 0
    };
    assert_eq(|_|"", x, 42);;

    let x = match opt {
        some(v) => v,
        none() => 0 // By a special syntax, you can omit the variable name for a variant of type `()`.
    };
    assert_eq(|_|"", x, 42);;

    let x = match opt {
        some(v) => v,
        _ => 0 // Any value can be matched by a variable pattern. 
                // Recall that `_` is NOT a special wildcard symbol, but just a variable name.
    };
    assert_eq(|_|"", x, 42);;

    pure()
);
```

## Traits

A Trait is a set of types. 
A trait is defined by a set of "methods" to be implemented by each member of it.

```
module Main;

// A Trait is a set of types. 
// A trait is defined by a set of "members" to be implemented by each member of it.

// `Greeter` is a set of types, where...
trait a : Greeter {
    // whose element has a member `greeting` that converts a value of type `a` into a greeting message greeting.
    greeting : a -> String;
}

// Let `I64` belong to the trait `MyToString`, where 
impl I64 : Greeter {
    // the `greeting` member is defined as follows.
    greeting = |n| "Hi! I'm a 64-bit integer " + n.to_string + "!";
}

/*
Traits are used for overloading operators.
For example, `Eq` trait is defined in standard library as follows: 

trait a : Eq {
    eq : a -> a -> Bool
}

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
    // In trait implementation, you can write type signatures for members. This improves code readability.
    mymap : (a -> b) -> Array a -> Array b;
    // Also, type variables (like `a` and `b`) defined in the member's type signature can be used in type annotations in the member's implementation.
    mymap = |f : a -> b, arr : Array a| (
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

## Associated types

Associated types can be thought of as type-level functions that take a trait (considered as a set of types) as their domain and return a new type.
A representative example is the `Iterator` trait in the standard library.

```
trait iter : Iterator {
    type Item iter;
    advance : iter -> Option (iter, Item iter);
}
```

Here, we define a type level function `Item`. 
`Item` takes an iterator type (i.e., a type implementing the `Iterator` trait) and returns the type of elements generated by it.

In a type signature of a function, you can write constraints on associated types.
For example, consider writing the type of a function that compares two iterators.
This function should take two iterators whose `Item`s are the same and implements the `Eq` trait.
Therefore, it has the following type:

```
is_equal : [iter1 : Iterator, iter2 : Iterator, Item iter1 = a, Item iter2 = a, a : Eq] iter1 -> iter2 -> Bool;
```

Associated type can have higher arity. 
The following is an example of defining type level function of arity 2 using associated types.

```
module Main;

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
    assert_eq(|_|"", (Nat::value : Value Zero).@data, 0);;
    assert_eq(|_|"", (Nat::value : Value One).@data, 1);;
    assert_eq(|_|"", (Nat::value : Value Two).@data, 2);;
    assert_eq(|_|"", (Nat::value : Value (Add One Two)).@data, 3);;
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

You can also define higher-kinded type alias. The following is an example of such type alias defined in `Std`:

```
type Lazy a = () -> a;
```

which defines a type alias `Lazy` of kind `* -> *`.

### Dynamic Iterators

In Fix, `Iterator` is a trait, and many types implement it. Therefore, there's no single "iterator" type; instead, each function that generates an iterator produces an iterator of a different type.

For example, the type of an iterator created from `Array a` by `to_iter` is `ArrayIterator a`, while the type of an iterator created by `range` is `CountUpIterator`. Additionally, applying `map` to an existing iterator creates an iterator of a more complex type. For instance, the type of `fib.to_iter.map(to_string)` in the tutorial's code example is `MapIterator (ArrayIterator I64) I64 String`.

This iterator design contributes significantly to improved performance. This is because the implementation of the `advance` function (a method of the `Iterator` trait) is uniquely determined by the iterator's type, allowing the compiler to perform optimizations such as inlining the `advance` function.

On the other hand, complex iterator types can be a hindrance to programming. For example, when defining a function that creates and returns an iterator, you would need to write a very complex iterator type for the function's return type. This becomes impossible, especially if the function returns an iterator created in a different way depending on the situation (its arguments).

To avoid the problem of complex iterator types, the following type is provided:

```
type DynIterator a = unbox struct { next: () -> Option (DynIterator a, a) };
```

`DynIterator` implements the `Iterator` trait.

You can use the `to_dyn` function to convert any iterator into a `DynIterator`.

```
// Converts an iterator into a dynamic iterator.
to_dyn : [iter : Iterator, Item iter = a] iter -> DynIterator a;
```

When defining a function that creates and returns an iterator, you can convert the complex iterator you've created into a `DynIterator` using `to_dyn` and then return it, simplifying the function's return type to a straightforward `DynIterator a`.

`DynIterator` is similar to Haskell's lazy lists. If you're porting beautiful Haskell code that uses lists to Fix, `to_dyn` might come in handy.

However, we recommend avoiding `DynIterator` if performance is critical. Here's one workaround for implementing a function that creates and returns an iterator while avoiding `DynIterator`.

As an example, consider the following function:

```
pythagorean_triples : I64 -> DynIterator (I64, I64, I64);
pythagorean_triples = |limit| (
    Iterator::range(1, limit+1).flat_map(|a| (
        Iterator::range(a, limit+1).flat_map(|b| (
            Iterator::range(b, limit+1).filter(|c| (
                a*a + b*b == c*c
            )).map(|c| (a, b, c))
        ))
    )).to_dyn
);
```

You don't need to understand the details of the code. Just note that it combines `range`, `flat_map`, `filter`, and `map` to create a complex iterator, which is then converted to a `DynIterator` using `to_dyn` before being returned.

To remove the `DynIterator` from this code, copy the code above into a text editor with the `fix` Language Server Protocol running. Next, hover your mouse over `to_dyn` to display its type. It should show something like this:

```
Std::Iterator::to_dyn : [a : Std::Iterator, Std::Iterator::Item a = b] a -> Std::Iterator::DynIterator b
Instantiated as:

(A very complex iterator type) -> Std::Iterator::DynIterator (Std::I64, Std::I64, Std::I64)
```

Since the display shows which iterator `to_dyn` is converting to a `DynIterator`, define that type as a type alias. Then, change the return type of `pythagorean_triples` to that type alias and remove `to_dyn`.

```
pythagorean_triples : I64 -> PythagorasIterator;
pythagorean_triples = |limit| (
    Iterator::range(1, limit+1).flat_map(|a| (
        Iterator::range(a, limit+1).flat_map(|b| (
            Iterator::range(b, limit+1).filter(|c| (
                a*a + b*b == c*c
            )).map(|c| (a, b, c))
        ))
    ))
);

type PythagorasIterator = (A very complex iterator type);
```

With this, you have successfully avoided `DynIterator`.

While this isn't the most elegant method, it's a practical way to avoid `DynIterator` for now. In a future version of Fix, we would like to enable writing code like this:

```
pythagorean_triples : I64 -> impl Iterator<Item = (I64, I64, I64)>;
pythagorean_triples = |limit| (
    Iterator::range(1, limit+1).flat_map(|a| (
        Iterator::range(a, limit+1).flat_map(|b| (
            Iterator::range(b, limit+1).filter(|c| (
                a*a + b*b == c*c
            )).map(|c| (a, b, c))
        ))
    ))
);
```

## Monads

### What is monad?

A Monad is a trait defined as follows:

```
trait [m : *->*] m : Monad {
    bind : (a -> m b) -> m a -> m b;
    pure : a -> m a;
}
```

In other words, a monad is a mapping `m` that creates a type from a type (like `Array` or `Option`), and it has two functions defined on it: `bind` and `pure`.

This is the only definition of a monad. To truly understand monads, it's crucial to know some examples. The following sections introduce three typical monads that are used in practice.

### Stateful Monads

A type that represents an "action (a computation that affects state)" is often a monad. We'll call such a monad a **"stateful monad."**

Consider the following definition:

```
type State s a = unbox struct { run : s -> (s, a) }
```

`State s` represents a computation that takes a value of type `s` (the "state") and returns a value of type `a` (the "result") along with a new state.

The following shows how to implement `State s : Monad` for any type `s`. Therefore, this `State s` provides an example of a stateful monad.

In a stateful monad, `bind` represents the combination of two actions. More specifically, the action `x.bind(f)` represents an action that does the following:

  * First, it executes action `x` (which updates the state). Let the result of action `x` be `r`.
  * Next, it executes action `f(r)` (which also updates the state).

The action `pure(v)` represents a computation that simply returns `v` without any interaction with the state.

To summarize, `State s : Monad` can be implemented as follows:

```
impl State s : Monad {
    bind = |f, x| State { run : |state| (
        let (state, r) = (x.@run)(state);
        (f(r).@run)(state)
    )};
    pure = |v| State { run : |state| (state, v) };
}
```

`IO`, as defined in Fix's standard library, is also an example of a stateful monad. `IO a` can be thought of as an "I/O action" that returns a value of type `a` while interacting with the computer's state. In fact, `IO` is defined as a wrapper for the type `IOState -> (IOState, a)`, where `IOState` should be imagined as a type that represents the "computer's state" (though it is actually defined as an empty struct).

Using `IO` as an example, let's see how `bind` works. `println(str) : IO ()` is an I/O action that prints `str` and a newline to standard output. `input_line : IO String` is an I/O action that reads a line from standard input and returns that line as a string. In this case, you can write an I/O action `echo1` that reads a line from standard input and prints it as-is, as follows:

```
echo1 : IO ();
echo1 = input_line.bind(|s| println(s));
```

#### Failure Monads

This type of monad represents a value that might have failed to compute.

In Fix's standard library, `Result` is defined as follows:

```
type Result e o = unbox union { ok : o, err: e };
```

`Result e o` contains either a value of type `o` or an error value of type `e`.

Another example is `Option`:

```
type Option a = union { none: (), some: a };
```

`Option a` represents a value that may or may not exist.

For any type `e`, `Result e` and `Option` implement `Monad`, providing examples of the failure monad.

In the failure monad, `bind` provides a way to perform short-circuit evaluation. `x.bind(f)` immediately returns a failure if `x` is a failure (or "none") value. The function `f` is called only if `x` is an "ok" (or "some") value `v`, and `x.bind(f)` becomes `f(v)`.

Additionally, `pure(v)` represents a successful computation with the value `v`.

Here is an example of the `Monad` implementation for `Option`:

```
impl Option : Monad {
    bind = |f, opt| match opt {
        none(_) => none(),
        some(v) => f(v)
    };
    pure = some;
}
```

As an example of `bind`'s usage, consider a function `add_opt : Option I64 -> Option I64 -> Option I64` that adds two integers wrapped in `Option`. This function is intended to perform the addition only if both are `some` values; otherwise, it should return `none`.

```
add_opt : Option I64 -> Option I64 -> Option I64;
add_opt = |x, y| (
    if x.is_none { none() };
    let x = x.as_some;
    if y.is_none { none() };
    let y = y.as_some;
    some(x+y)
);
```

Using `bind` for `Option`, the function above can be written concisely as:

```
add_opt : Option I64 -> Option I64 -> Option I64;
add_opt = |x, y| x.bind(|x| y.bind(|y| some(x+y)));
```

### Sequence Monads

Types that represent a sequence of elements, like an array, can also be instances of a monad. In the Fix standard library, both `Array` and `DynIterator` implement the `Monad` trait.

In a sequence monad, `[x, y, z, ...].bind(f)` represents `f(x) + f(y) + f(z) + ...`, where `+` denotes the concatenation of two sequences. This `bind` operation is often called **"flat\_map"** in other languages.

The `pure(x)` operation represents a single-element sequence `[x]`.

For example, consider a function `product : Array a -> Array b -> Array (a, b)` that computes the Cartesian product. It can be implemented like this:

```
product : Array a -> Array b -> Array (a, b);
product = |xs, ys| xs.bind(|x| ys.bind(|y| pure $ (x, y)));
```

If we let `xs == [x0, x1, ...]` and `ys == [y0, y1, ...]`, we can see how `product(xs, ys)` expands to compute the Cartesian product:

```
xs.bind(|x| ys.bind(|y| pure $ (x, y)))
== ys.bind(|y| pure $ (x0, y)) + ys.bind(|y| pure $ (x1, y)) + ...
== (pure $ (x0, y0)) + (pure $ (x0, y1)) + ... + (pure $ (x1, y0)) + (pure $ (x1, y1)) + ... + ...
== [(x0, y0)] + [(x0, y1)] + ... + [(x1, y0)] + [(x1, y1)] + ... + ...
== [(x0, y0), (x0, y1), ..., (x1, y0), (x1, y1), ..., ...]
```

### `do` Blocks and the monadic bind operator `*`

The Fix prefix unary operator `*` provides a more concise way to use `bind`. The code `B(*x)` is expanded to `x.bind(|v| B(v))`.

Here, `B(*x)` is the smallest **do block** that encloses the expression `*x`. A do block is created either explicitly or implicitly as follows:

  - You can explicitly create a do block with `do { ... }`.
  - A global definition `name = ...` implicitly defines a do block `...`.
  - A let definition `let name = val (in|;) ...` implicitly defines a do block `...`.
  - A lambda expression `|arg| ...` implicitly defines a do block `...`.
  - An if expression `if cond { ... } else { ... }` implicitly defines two do blocks `...`.
  - A match expression `match val { pat => ... }` implicitly defines a do block `...`.
  - The double semicolon syntax (explained later) `act;; ...` implicitly defines a do block `...`.

In a previous section, we showed an example of creating `echo1 : IO ()` from `input_line : IO String` and `println : String -> IO ()` using `bind` in the stateful monad `IO`.

```
echo1 : IO ();
echo1 = input_line.bind(|s| println(s));
```

Using the `*` operator for a more concise way to use `bind`, the code above can be written as:

```
echo1 : IO ();
echo1 = println(*input_line);
```

This can be interpreted as the `*` operator extracting the content of the `input_line` monad value and passing that content to `println`. In fact, writing it like this is the same:

```
echo1 : IO ();
echo1 = (
    let s = *input_line;
    println(s)
);
```

Similarly,

```
add_opt : Option I64 -> Option I64 -> Option I64;
add_opt = |x, y| x.bind(|x| y.bind(|y| Option::some(x+y)));
```

can be written as:

```
add_opt : Option I64 -> Option I64 -> Option I64;
add_opt = |x, y| some $ *x + *y;
```

Here again, the `*` operator takes the content of the monad values `x` and `y`, adds them, and passes the result to `some` to create the final `Option I64` value.

```
product : Array a -> Array b -> Array (a, b);
product = |xs, ys| xs.bind(|x| ys.bind(|y| pure $ (x, y)));
```

can be written as:

```
product : Array a -> Array b -> Array (a, b);
product = |xs, ys| pure $ (*xs, *ys);
```

Here, `*xs` and `*ys` take one element at a time from each sequence and pass the pair to `pure` to compute the Cartesian product.

### When an explicit `do` block is needed

In the examples so far, you didn't need to explicitly create a `do` block when using the `*` operator. Here's an example where you do.

```
add_opt_unwrap : Option I64 -> Option I64 -> I64;
add_opt_unwrap = |x, y| do { some $ *x + *y }.as_some;
```

This function adds two `Option I64` values and returns the result as an `I64`. If either value is `none`, the program will halt.

The `add_opt_unwrap` definition above expands to the following and compiles successfully:

```
add_opt_unwrap = x.bind(|x| y.bind(|y| some $ x + y)).as_some;
```

On the other hand, if you don't create an explicit `do` block, like this:

```
add_opt_unwrap : Option I64 -> Option I64 -> I64;
add_opt_unwrap = |x, y| (some $ *x + *y).as_some;
```

it expands to:

```
add_opt_unwrap = |x, y| x.bind(|x| y.bind(|y| (some $ x + y).as_some));
```

The latter code, which doesn't use `do`, results in a type error and won't compile. In fact, the return value of the outer `bind` is of type `Option I64`, but the function `add_opt_unwrap` is required to return an `I64`.

While this may seem complex, once you get the feel that "the scope of a `do` block becomes a monad value," it becomes relatively easy to decide whether you need to explicitly create a `do` block when using `*`.

The code that compiles successfully:

```
add_opt_unwrap : Option I64 -> Option I64 -> I64;
add_opt_unwrap = |x, y| do { some $ *x + *y }.as_some;
```

is fine because the explicit `do`'s scope becomes an `Option I64` monad, and you can then apply `as_some` to it.

If we explicitly show the scope of the implicitly created `do` block for the code that fails to compile:

```
add_opt_unwrap : Option I64 -> Option I64 -> I64;
add_opt_unwrap = |x, y| (some $ *x + *y).as_some;
```

it would look like this:

```
add_opt_unwrap : Option I64 -> Option I64 -> I64;
add_opt_unwrap = |x, y| do { (some $ *x + *y).as_some };
```

Here, the scope of the `do` block becomes a value of type `Option I64`, but this function needs to return an `I64`, causing a type error.

### Chaining monadic actions with the `;;` Syntax

The function `println : String -> IO ()` creates an IO action that takes a string and prints it to standard output. If you want to perform `println` multiple times, you can use the `*` operator as shown below:

```
module Main;

main : IO ();
main = (
    let _ = *println("The sum of 1 + 2 is: ");
    let _ = *println((1 + 2).to_string);
    pure()
);
```

Since the result of the `println(...)` IO action isn't needed, we assign it to the variable `_` to ignore it. Additionally, `pure() : IO ()` represents a "do nothing" IO action.

The double-semicolon syntax `{expr0};; {expr1}` is equivalent to `let _ = *{expr0}; {expr1}`. Therefore, the code above can be written like this:

```
module Main;

main : IO ();
main = (
    println("The sum of 1 + 2 is: ");;
    println((1 + 2).to_string);;
    pure()
);
```

### Fix's Iterator is not a monad

As previously mentioned, types that represent a sequence of elements often become "sequence monads." However, `Iterator` is not a type in the Fix standard library but a trait, so `Iterator` itself is not a monad.

Among the iterators defined in `Std`, only `DynIterator` implements the `Monad` trait, making it a sequence monad. Therefore, you can use the `*` operator to manipulate a `DynIterator`.

The following program finds and lists all Pythagorean triples `(a, b, c)` that satisfy the condition `1 <= a <= b <= c <= limit`. The `to_dyn` method is used to convert an iterator created by `range(a, b)` into a `DynIterator`.

```
pythagorean_triples : I64 -> DynIterator (I64, I64, I64);
pythagorean_triples = |limit| (
    let a = *Iterator::range(1, limit+1).to_dyn;
    let b = *Iterator::range(a, limit+1).to_dyn;
    let c = *Iterator::range(b, limit+1).to_dyn;
    if a*a + b*b != c*c {
        DynIterator::empty
    };
    (a, b, c).pure
);
```

As stated in [Dynamic Iterators](#dynamic-iterators), `DynIterator` has inferior performance compared to other iterators. Therefore, here's how to rewrite the code above without using `DynIterator`.

As previously mentioned, `bind` in a sequence monad is known as the "flat map" operation. Fix's standard library provides `flat_map` for iterators. By recalling the definition of the `*` operator, rewriting the code above using explicit `bind`, and then replacing `bind` with `flat_map`, we can get a version of the code that doesn't use `DynIterator`.

The result is as follows. Since the result of an iterator computation can have a very complex type, we use the `to_array` method at the end to convert it into an array.

```
pythagorean_triples : I64 -> Array (I64, I64, I64);
pythagorean_triples = |limit| (
    Iterator::range(1, limit+1).flat_map(|a| (
        Iterator::range(a, limit+1).flat_map(|b| (
            Iterator::range(b, limit+1).filter(|c| (
                a*a + b*b == c*c
            )).map(|c| (a, b, c))
        ))
    )).to_array
);
```

## Foreign Function Interface (FFI)

By linking a static or shared library to a Fix program using the `--static-link` (`-s`) or `--dynamic-link` (`-s`) compiler flags, you can call native functions within a Fix program and call Fix functions within a library.

However, using FFI can allow external functions to break Fix's guarantees of immutability and memory safety. Programmers are responsible for hiding the side effects of external functions in `IO` and properly managing resources to avoid segmentation faults and memory leaks.

-----

### Calling External Functions from Fix

To call an external function from Fix, you use the `FFI_CALL(_IO|_IOS)[...]` expression. The syntax is as follows:

```
FFI_CALL[{function_signature}, {arg_0}, {arg_1}, ...]
```

```
FFI_CALL_IO[{function_signature}, {arg_0}, {arg_1}, ...]
```

```
FFI_CALL_IOS[{function_signature}, {arg_0}, {arg_1}, ..., {iostate}]
```

Use `FFI_CALL` to call a pure external function. `FFI_CALL[...]` takes the same arguments as the external function and returns a Fix value corresponding to the external function's return value.

If the external function has side effects, use `FFI_CALL_IO`, which returns an `IO` monad value.

You can also use `FFI_CALL_IOS` instead of `FFI_CALL_IO`. This function takes an additional argument of type `IOState` and returns a value of type `(IOState, a)`, where `a` is the return type of the external function.

Note: `IOState` is a type defined in the Fix standard library that represents the internal state of the `IO` monad. In fact, `IO` is defined as follows:

```
type IO a = unbox struct { runner : IOState -> (IOState, a) };
```

As an example of `FFI_CALL` and `FFI_CALL_IO` usage, here is the implementation of `Std::consumed_time_while_io`.

```
// Gets the elapsed clock (CPU time) while an I/O action is running.
consumed_time_while_io : IO a -> IO (a, F64);
consumed_time_while_io = |io| (
    let s = *FFI_CALL_IO[I64 fixruntime_clock()];
    let r = *io;
    let t = *FFI_CALL_IO[I64 fixruntime_clock()];
    let t = FFI_CALL[F64 fixruntime_clocks_to_sec(I64), t - s];
    pure $ (r, t)
);
```

`fixruntime_clock` and `fixruntime_clocks_to_sec` are C language functions defined in the Fix runtime library.

Because `fixruntime_clock` is a function with side effects, it's called using `FFI_CALL_IO`. In contrast, `fixruntime_clocks_to_sec` is a pure function, so it's called using `FFI_CALL`.

In the `{c_function_signature}` of `FFI_CALL` (or `FFI_CALL_IO`, `FFI_CALL_IOS`), you specify the name and signature of the external function to be called. The signature is written in the format `{return_type} {function_name}({arg_type_0}, {arg_type_1}, ...)`.

The following types can be used for `{return_type}` or `{arg_type_i}`:

* Pointers: `Ptr`
* Numeric types with explicit bit widths: `I8`, `U8`, `I16`, `U16`, `I32`, `U32`, `I64`, `U64`, `F32`, `F64`
* C numeric types: `CChar`, `CUnsignedChar`, `CShort`, `CUnsignedShort`, `CInt`, `CUnsignedInt`, `CLong`, `CUnsignedLong`, `CLongLong`, `CUnsignedLongLong`, `CSizeT`, `CFloat`, `CDouble`
* Substitute for `void`: `()`

### Exporting Fix Values and Functions to External Languages

To use a Fix value from an external language, you use the `FFI_EXPORT[{fix_value_name}, {c_function_name}];` syntax.

```
fix_increment : CInt -> CInt;
fix_increment = |x| x + 1.c_int;
FFI_EXPORT[fix_increment, increment]; // Defines the function `int increment(int);`
```

For example, to call the `fix_increment` function from a C library, you would declare `int increment(int);` in your C source code and call `increment` where needed.

The signature of the exported function is automatically determined by the type of the Fix value. The following examples show how the C function signature is determined from the Fix value's type.

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

### Managing External Resources in Fix

Some C functions allocate resources that must eventually be freed by another C function. The most famous examples are `malloc` / `free` and `fopen` / `fclose`. If you use `FFI_CALL` from Fix to allocate a resource, you must call the freeing function again using `FFI_CALL` at the end of that resource's lifetime.

To manage such resources, you can use `Std::FFI::Destructor`. A `Destructor a` is a boxed type that, as its data, holds a `value` of type `a` and a `dtor` of type `a -> IO a`. When the Fix compiler deallocates a `Destructor a` from heap memory, it calls `dtor` on `value`.

A typical use case is to store a pointer to a resource obtained with `malloc` or `fopen` in the `value` field of a `Destructor Ptr` and store the IO operation that calls `free` or `fclose` in the `dtor` field. This ensures the resource is automatically freed when the `Destructor Ptr` value goes out of scope.

However, using `Destructor` properly is not easy and requires attention to various details. Please also check the functions in the documentation for [`Destructor`](https://www.google.com/search?q=/std_doc/Std.md%23Destructor) and [namespace Destructor](https://www.google.com/search?q=/std_doc/Std.md%23namespace_Std::FFI::Destructor).

### Managing ownership of Fix's boxed value in a foreign language

In the previous section, we explained how to manage resources allocated by C functions (such as `fopen`) in Fix.
This section explains the reverse: how to manage Fix's boxed type values in a foreign language such as C.

The function `Std::FFI::boxed_to_retained_ptr : [a : Boxed] a -> IO Ptr` returns a pointer to a Fix boxed type value.
Also, `Std::FFI::boxed_from_retained_ptr : [a : Boxed] Ptr -> IO a` creates a Fix boxed type value from such a pointer.
Naturally, when creating a boxed type value from a pointer, you must correctly specify (using type annotations, etc.) the type of the value that the pointer points to. Otherwise, the behavior is undefined.

By converting a boxed type value to a pointer, you can pass Fix values to a foreign language.
Then, by exporting a Fix function that applies `boxed_from_retained_ptr` to the passed pointer and uses the resulting value to perform some processing,
you can enable the foreign language to use Fix values.

```
create_fix_array : IO Ptr;
create_fix_array = (
    let arr = [1,2,3,4,5];
    arr.boxed_to_retained_ptr
);
FFI_EXPORT[create_fix_array, create_fix_array]; // void* create_fix_array(void); can be called from C.

get_fix_array_element : Ptr -> I64 -> IO I64;
get_fix_array_element = |ptr, idx| (
    let arr : Array I64 = *boxed_from_retained_ptr(ptr);
    pure $ arr.@(idx)
);
FFI_EXPORT[get_fix_array_element, get_fix_array_element]; // int64_t get_fix_array_element(void* ptr, int64_t idx); can be called from C.
```

The lifetime of Fix's boxed type values is managed by a reference counter.
Normally, the increment and decrement of the reference counter is done automatically by the Fix compiler,
but when you create a pointer from a boxed type value using `boxed_to_retained_ptr`, you must do **exactly one** of the following:
- Decrement the reference counter
- Return the responsibility of decrementing the reference counter to the Fix compiler

To decrement the reference counter, first call `Std::FFI::get_funptr_release : a -> Ptr` from the foreign language side to get a function pointer of type `void (*)(void*)`.
By calling this function pointer (passing the pointer to the value as an argument), you can decrement the reference counter.

To return the responsibility of decrementing the reference counter to the Fix compiler, pass the pointer to `boxed_from_retained_ptr`.

Therefore, when using a Fix value **at most once** on the foreign language side, implement as follows:
- Use `boxed_to_retained_ptr` to get a pointer and pass it to the foreign language.
- When using the value on the foreign function side, pass the pointer to the Fix side and do the work using `boxed_from_retained_ptr`.
- When not using the value on the foreign function side, call the function obtained with `get_funptr_release` from the foreign language side to decrement the reference counter.

However, you may want to use a Fix value multiple times on the foreign language side.
For such cases, there is a way to increase "the number of times you should fulfill (or return) the responsibility".
To do this, call `Std::FFI::get_funptr_retain : a -> Ptr` from the foreign language side to get a function pointer of type `void (*)(void*)`, and call that function pointer.

Therefore, when using a Fix value **multiple times** on the foreign language side, implement as follows:
- Use `boxed_to_retained_ptr` to get a pointer and pass it to the foreign language.
- When using the value on the foreign function side, call the function obtained with `get_funptr_retain`, then pass the pointer to the Fix side and do the work using `boxed_from_retained_ptr`.
- When the value is no longer needed on the foreign function side, call the function obtained with `get_funptr_release` to decrement the reference counter.

For example, suppose you use a Fix value N times on the foreign language side.
"The number of times you should fulfill (or return) the responsibility" is initially 1, but since `get_funptr_retain` is called N times, it becomes N+1 in total.
Since `boxed_from_retained_ptr` is called N times, the number of times the responsibility is returned to the Fix side is N times.
The remaining 1 responsibility is fulfilled by calling `get_funptr_release` on the foreign language side.

If the "foreign language" is C++, you can automate the above management by implementing a class that wraps Fix values as follows:
- Member variable
    - `void* ptr;`. Pointer to a Fix boxed type value.
- Constructor
    - Call a Fix function. In that Fix function, `boxed_to_retained_ptr` is performed to get a pointer, and that pointer is stored in the member variable `ptr`.
- Copy constructor
    - Call the function pointer obtained with `Std::FFI::get_funptr_retain` (type `void (*)(void*)`) with `ptr`.
- Destructor
    - Call the function pointer obtained with `Std::FFI::get_funptr_release` (type `void (*)(void*)`) with `ptr`.
- Member functions
    - Call a Fix function with `ptr`. On the Fix side, `boxed_from_retained_ptr` is performed to convert the pointer to a Fix value, and that value is used.

There are also some things to note in the implementation on the Fix side.
- The pointer obtained with `boxed_to_retained_ptr` must be passed to the foreign language side exactly once.
    - To achieve this, processing that exchanges pointers with foreign languages should use `IO`.
        - When calling a Fix function from the foreign language to request a pointer, that Fix function should be an `IO` action (like `create_fix_array` above). By executing the `IO` action returned by `boxed_to_retained_ptr` in it, it is guaranteed that the created pointer is passed to the foreign language only once.
        - When calling an external function from the Fix side to pass a pointer, you should use `FFI_CALL_IO` instead of `FFI_CALL`. Also, you should execute the `IO` action returned by `boxed_to_retained_ptr` and `FFI_CALL_IO` by connecting them with `bind`. If you want to pass a pointer to the foreign language in a pure context, execute that connected `IO` action with `unsafe_perform`.
    - If you don't follow the above, the optimization by the Fix compiler may cause the number of calls to `boxed_to_retained_ptr` and the number of times the pointer is passed to the foreign language to not match.
- The pointer received from the foreign language must be passed to `boxed_from_retained_ptr` exactly once.
    - To achieve this, the Fix function that is called from the foreign language and receives a pointer should be an `IO` action (like `get_fix_array_element` above). By executing the `IO` action returned by `boxed_from_retained_ptr` in it, it is guaranteed that the received pointer is converted to a Fix value only once.
    - If you don't follow the above, the optimization by the Fix compiler may cause the number of times the pointer is received from the foreign language and the number of calls to `boxed_from_retained_ptr` to not match.
- Fix's reference counting is not thread-safe by default. Therefore, if you distribute a pointer received from Fix to multiple threads on the foreign language side, data races may occur regarding the increment and decrement of the reference counter.
    - To avoid this, call `Std::mark_threaded : a -> a` on the boxed type value before calling `boxed_to_retained_ptr` to set the reference counting to thread-safe mode.

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

If you want to access to the fields `x` and `y` of Fix's object `vec` from C side, `Std::FFI::borrow_boxed : (Ptr -> b) -> a -> b` will be useful: 
`vec.borrow_boxed(|p| FFI_CALL[() access_vec(Ptr), p])` will allows `access_vec` on work on `vec.@x` and `vec.@y`.

NOTE: 
At least in the current version of Fix, the memory layout of Fix's struct is determined by the default behaviour of LLVM, and as long as I know it is equivalent to C's struct memory layout. 
In a future version, the situation may be changed. I may introduce a specifier (suppose it is written as `expr_c`) for a programmer to assure that the layout is equivalent to C, and the struct layout with no `expr_c` specifier may be optimized (e.g., reorder field ordering).

## `eval` syntax

The expression `eval {expr0}; {expr1}` evaluates both `{expr0}` and `{expr1}`, and returns the value of `{expr1}`.

Fix may omit the evaluation of unnecessary expressions during optimization. For example, in a program like:
```
main : IO () = (
    let x = 1 + 2;
    println("Hello, World!");
);
```
the evaluation of `x = 1 + 2` does not affect the program's behavior, so the Fix compiler may omit this evaluation.

The `eval` syntax is used to instruct the Fix compiler not to omit the evaluation of expressions.

This syntax is primarily used for debugging purposes.
For example, `debug_eprint : String -> ()` is a function that outputs a message to standard error without using the `IO` monad.
This function should be used with the `eval` syntax like:
```
my_add : I64 -> I64 -> I64 = |x, y| (
    let z = x + y
    eval debug_eprint("The sum is: " + z.to_string);
    z
);
```
In this example, the call to `debug_eprint(...)` does not affect the result of `my_add`, but using `eval` guarantees that the message will be output.

Notes:
- If a program does not use the result of the entire `eval` expression (i.e., the result of `{expr1}`), the Fix compiler may omit the entire `eval` expression, resulting in `{expr0}` not being evaluated.
- Currently, the evaluation order of `{expr0}` and `{expr1}` is not guaranteed.
- As long as the `eval` expression is necessary for the program execution, the compiler guarantees that `{expr0}` will be evaluated at least once, but it does not guarantee how many times it will be evaluated. For example:
```
truth : I64 = eval debug_println("evaluated"); 42;
```
For code like this, there is no guarantee whether "evaluated" will be output every time `truth` is referenced, or only once when it is first referenced.

## Substitute Pattern

This section explains a phenomenon that can be considered one of Fix's weaknesses and the "substitute pattern" as a workaround.

Consider a situation where you have the following type definition:
```
type MyType = struct {
    field1: Array I64,
    field2: Array I64,
    ... // many other fields
};
```

Also, suppose you have the following function:
```
modify_array : Array I64 -> Array I64;
```

Assume that `modify_array` is implemented to modify the given `Array I64` in-place (when it is unique).

If you have a value `x` of type `MyType` and want to modify its `field1` with `modify_array`, you can write:
```
x.mod_field1(modify_array)
```

When this code is executed on an `x` with a unique `field1`, the `field1` is modified in-place as expected.

Next, consider the following function:
```
modify_arrays : (Array I64, Array I64) -> (Array I64, Array I64);
```

This function is also assumed to be implemented to modify the two given `Array I64` values in-place (when they are unique).

Now, if you want to modify both `field1` and `field2` of `x: MyType` simultaneously with `modify_arrays`, how should you write it?

There is no built-in function like `mod_field1_and_field2`.
Therefore, you might consider writing something like this:
```
let (field1, field2) = modify_arrays((x.@field1, x.@field2));
x.set_field1(field1).set_field2(field2)
```

However, in this code, even if `field1` and `field2` of `x` were originally unique, copies may be created inside `modify_arrays`.
This is because the name `x` is still used after `modify_arrays`.
As a result, `x` and the array values reachable from it must remain unchanged even **after** the call to `modify_arrays`,
which means `modify_arrays` is not allowed to directly edit the memory regions pointed to by `x.@field1` and `x.@field2`.

One way to avoid this problem is to temporarily exchange `field1` and `field2` of `x` with substitute values (here we use empty arrays), and pass the extracted arrays to `modify_arrays`.
```
let (x, arr1) = x[^field1].ixchg([]); // exchange field1 with an empty array
let (x, arr2) = x[^field2].ixchg([]); // exchange field2 with an empty array
let (arr1, arr2) = modify_arrays((arr1, arr2));
x.set_field1(arr1).set_field2(arr2) // restore field1 and field2 to the original x
```

This method is only effective when there exists a value that can be used as a "substitute value" for the type of `field1` and `field2` (here `Array I64`) (here, the empty array `[]`).
If this is not the case, consider changing the type of `field1` and `field2` to an `Option` type or similar, and using `none()` as the substitute value.

## Operator and Syntax Precedence

The precedence and associativity of operators and some syntax in Fix are shown in the following table.

* Operators with higher precedence bind more tightly.
    * For example, `*` (binary operator) has higher precedence than `+`, so the expression `x + y * z` is interpreted as `x + (y * z)`.
* When multiple operators with the same precedence appear, they bind according to their associativity.
    * For example, `+` (binary operator) and `-` (binary operator) have the same precedence and are left-associative, so the expression `x - y + z` is interpreted as `(x - y) + z`.
* The index syntax `[...]` has special associativity.
    * Both `x[i]` and `x[i][j]` are valid expressions, but `x[i][j]` does not match `(x[i])[j]`.
    * Also, `x[i][j]` does not match `x([i][j])` (in fact, `[i][j]` is not an expression).
    * `x` and the consecutive sequence of `[...]` that follows form a single expression.
* The index syntax `[...]` and the dot syntax `.` have the same precedence and are left-associative.
    * Therefore, `x.f[i].iget` is interpreted as `((x.f)[i]).iget`.

<table>
    <thead>
        <tr>
            <th>Precedence</th>
            <th>Symbol</th>
            <th>Type</th>
            <th>Associativity</th>
            <th>Description</th>
        </tr>
    </thead>
    <tbody>
        <tr>
            <td>11</td>
            <td>(...)</td>
            <td>postfix</td>
            <td>left</td>
            <td>Function application: <code>f(x,y)</code></td>
        </tr>
        <tr>
            <td>10</td>
            <td>[...]</td>
            <td>postfix</td>
            <td>special</td>
            <td>Index syntax: <code>x[i][^field]</code> = <code>|a| x.(act_at_index(i) $ act_field $ a)</code></td>
        </tr>    
        <tr>
            <td>10</td>
            <td>.</td>
            <td>binary</td>
            <td>left</td>
            <td>Right-to-left function application: <code>x.f</code> = <code>f(x)</code></td>
        </tr>
        <tr>
            <td>9</td>
            <td>*</td>
            <td>prefix</td>
            <td>right</td>
            <td>Monadic bind: <code>Std::Monad::bind</code></td>
        </tr>
        <tr>
            <td>8</td>
            <td>&lt;&lt;</td>
            <td>binary</td>
            <td>left</td>
            <td>Right-to-left function composition: <code>g &lt;&lt; f</code> = <code>|x| g(f(x))</code> (<code>Std::compose</code>)</td>
        </tr>
        <tr>
            <td>8</td>        
            <td>&gt;&gt;</td>
            <td>binary</td>
            <td>left</td>
            <td>Left-to-right function composition: <code>(f &gt;&gt; g)(x)</code> = <code>|x| g(f(x))</code> (<code>Std::compose</code>)</td>
        </tr>
        <tr>
            <td>7</td>        
            <td>-</td>
            <td>prefix</td>
            <td>right</td>
            <td>Negation of number: <code>Std::Neg::neg</code></td>
        </tr>
        <tr>
            <td>7</td>        
            <td>!</td>
            <td>prefix</td>
            <td>right</td>
            <td>Logical NOT: <code>Std::Not::not</code></td>
        </tr>
        <tr>
            <td>6</td>        
            <td>*</td>
            <td>binary</td>
            <td>left</td>
            <td>Multiplication of numbers: <code>Std::Mul::mul</code></td>
        </tr>
        <tr>
            <td>6</td>        
            <td>/</td>
            <td>binary</td>
            <td>left</td>
            <td>Division of numbers: <code>Std::Div::div</code></td>
        </tr>
        <tr>
            <td>6</td>        
            <td>%</td>
            <td>binary</td>
            <td>left</td>
            <td>Remainder of division: <code>Std::Rem::rem</code></td>
        </tr>
        <tr>
            <td>5</td>        
            <td>+</td>
            <td>binary</td>
            <td>left</td>
            <td>Addition of numbers: <code>Std::Add::add</code></td>
        </tr>
        <tr>
            <td>5</td>        
            <td>-</td>
            <td>binary</td>
            <td>left</td>
            <td>Subtraction of numbers: <code>Std::Sub::sub</code></td>
        </tr>
        <tr>
            <td>4</td>        
            <td>==</td>
            <td>binary</td>
            <td>left</td>
            <td>Equality comparison: <code>Std::Eq::eq</code></td>
        </tr>
        <tr>
            <td>4</td>        
            <td>!=</td>
            <td>binary</td>
            <td>left</td>
            <td><code>x != y</code> = <code>!(x == y)</code></td>
        </tr>
        <tr>
            <td>4</td>        
            <td>&lt;=</td>
            <td>binary</td>
            <td>left</td>
            <td>Less-than-or-equal-to comparison: <code>Std::LessThanOrEq::less_than_or_eq</code></td>
        </tr>
        <tr>
            <td>4</td>        
            <td>&gt;=</td>
            <td>binary</td>
            <td>left</td>
            <td><code>x &gt;= y</code> = <code>y &lt;= x</code></td>
        </tr>
        <tr>
            <td>4</td>        
            <td>&lt;</td>
            <td>binary</td>
            <td>left</td>
            <td>Less-than comparison: <code>Std::LessThan::less_than</code></td>
        </tr>
        <tr>
            <td>4</td>        
            <td>&gt;</td>
            <td>binary</td>
            <td>left</td>
            <td><code>x &gt; y</code> = <code>y &lt; x</code></td>
        </tr>
        <tr>
            <td>3</td>        
            <td>&amp;&amp;</td>
            <td>binary</td>
            <td>right</td>
            <td>Short-circuit logical AND: <code>x &amp;&amp; y</code> = <code>if x { y } else { false }</code></td>
        </tr>
        <tr>
            <td>2</td>        
            <td>||</td>
            <td>binary</td>
            <td>right</td>
            <td>Short-circuit logical OR: <code>x || y</code> = <code>if x { true } else { y }</code></td>
        </tr>
        <tr>
            <td>1</td>
            <td>$</td>
            <td>binary</td>
            <td>right</td>
            <td>Right-associative function application: <code>f $ g $ x</code> = <code>f(g(x))</code></td>
        </tr>
        <tr>
            <td>0</td>
            <td>;;</td>
            <td>binary</td>
            <td>right</td>
            <td>Conjunction of monadic actions: <code>m0;; m1</code> = <code>let _ = *m0; m1</code></td>
        </tr>
    </tbody>
</table>


# Compiler features

## Project file

A project file is a TOML file which contains information about a Fix project, such as: 

- The project name, version or author, etc.,
- Which Fix source files are included in the project,
- Dependencies to the other Fix projects,
- Non-Fix programs (such as object files, static or dynamic libraries) to be linked,
- Commands to be executed before the compilation.

The project file should have a name "fixproj.toml".
Many of features of "fix" command tries to read the project file in the current directory, and if found, uses the information in it.
Moreover, some subcommands (e.g., "fix deps", "fix docs" or "fix language-server") requires the project file to be present.

"fix init" command generates [a template project file](./src/docs/project_template.toml).
This file contains a list of available fields (almost all of which are commented out by default), examples, and explanations, so please refer to it.

### About Duplicated Build Settings

When building a Fix program, settings can be specified in the following locations:

(In order of priority, from highest to lowest)
- Compiler options
- `build.test` section of the project file ("fixproj.toml")
    - This only affects the `fix test` command.
- `build` section of the project file ("fixproj.toml")
- `build` section of the dependent library's project file ("fixproj.toml")

When the same setting is specified in multiple locations, how the final setting value is determined varies depending on the setting item. There are the following patterns:
- (Overwrite) Overwritten by the setting in the location with higher priority (e.g., optimization level).
- (Merge) All setting values are merged (e.g., list of source files).

Additionally, some settings specified in a dependent library's project file have no effect on the main project's build at all.

The following table shows how each setting is handled.

- Field: Field name in "fixproj.toml"
- Option: Option name for the `fix` command
- Type: Whether settings with higher priority overwrite or are merged
- Dependent Project: Whether settings in a dependent library's project file affect the main project's build

<table>
    <thead>
        <tr>
            <th>Field</th>
            <th>Option</th>
            <th>Type</th>
            <th>Dependent Project</th>
            <th>Description</th>
        </tr>
    </thead>
    <tbody>
        <tr>
            <td>files</td>
            <td>-f, --file</td>
            <td>Merge</td>
            <td>Affects</td>
            <td>Source files</td>
        </tr>
        <tr>
            <td>objects</td>
            <td>-b, --object</td>
            <td>Merge</td>
            <td>Affects</td>
            <td>Object files to be linked</td>
        </tr>
        <tr>
            <td>static_links</td>
            <td>-s, --static-link</td>
            <td>Merge</td>
            <td>Affects</td>
            <td>Static libraries to be linked</td>
        </tr>
        <tr>
            <td>dynamic_links</td>
            <td>-d, --dynamic-link</td>
            <td>Merge</td>
            <td>Affects</td>
            <td>Dynamic libraries to be linked</td>
        </tr>
        <tr>
            <td>library_paths</td>
            <td>-L, --library-paths</td>
            <td>Merge</td>
            <td>Affects</td>
            <td>Library search paths</td>
        </tr>
        <tr>
            <td>ld_flags</td>
            <td>--ld-flags</td>
            <td>Merge</td>
            <td>Affects</td>
            <td>Linker flags</td>
        </tr>
        <tr>
            <td>preliminary_commands</td>
            <td>None</td>
            <td>Merge</td>
            <td>Affects</td>
            <td>Commands to be executed before compilation</td>
        </tr>
        <tr>
            <td>threaded</td>
            <td>--threaded</td>
            <td>Merge (OR)</td>
            <td>Affects</td>
            <td>Enable multi-threading</td>
        </tr>
        <tr>
            <td>debug</td>
            <td>-g, --debug</td>
            <td>Merge (OR)</td>
            <td>Does not affect</td>
            <td>Generate debugging information</td>
        </tr>
        <tr>
            <td>opt_level</td>
            <td>-O, --opt-level</td>
            <td>Overwrite</td>
            <td>Does not affect</td>
            <td>Optimization level</td>
        </tr>
        <tr>
            <td>output</td>
            <td>-o, --output</td>
            <td>Overwrite</td>
            <td>Does not affect</td>
            <td>Output file path</td>
        </tr>
        <tr>
            <td>output_type</td>
            <td>--output-type</td>
            <td>Overwrite</td>
            <td>Does not affect</td>
            <td>Output file type (executable/dynamic library)</td>
        </tr>
        <tr>
            <td>backtrace</td>
            <td>--backtrace</td>
            <td>Merge (OR)</td>
            <td>Does not affect</td>
            <td>Generate backtrace</td>
        </tr>
        <tr>
            <td>disable_cpu_features</td>
            <td>--disable-cpu-feature</td>
            <td>Merge</td>
            <td>Does not affect</td>
            <td>CPU features to be disabled</td>
        </tr>
        <tr>
            <td>no_runtime_check</td>
            <td>--no-runtime-check</td>
            <td>Merge (OR)</td>
            <td>Does not affect</td>
            <td>Disable runtime checks</td>
        </tr>
    </tbody>
</table>

Note:
For some settings, such as optimization level and debugging information generation, you might expect that settings in dependent projects would take effect only within the scope of those dependent projects, rather than not affecting the main project at all.
However, the Fix build system currently does not have a mechanism to separate compilation units on a per-dependent-project basis, so settings like optimization level and debugging information generation use a single value across the entire build.

## Managing dependencies

Dependencies of a Fix project are represented by [[dependencies]] elements in the "fixproj.toml" file.
The following is an example of adding two dependencies: "hash" in the remote repository and "mylib" in the local repository.

```
[[dependencies]]
name = "hash"
version = "0.1.0"
git = { url = "https://github.com/tttmmmyyyy/fixlang-hash.git" }

[[dependencies]]
name = "mylib"
version = "*"
path = "/path/to/mylib"
```

Here, the notation `version = "0.1.0"` means that it requires version "0.1.0" or other versions that are SemVer compatible with it.
The definition of SemVer compatibility is the same as that of Cargo. See https://doc.rust-lang.org/cargo/reference/resolver.html#semver-compatibility for details.

You can add dependencies manually by adding [[dependencies]] elements, or by using "fix deps add {name}@{ver-req}" command.
The "fix deps add" command searches the specified Fix project from "registry file"s, and add the dependency to the project file if it is found.
The default registry file is managed in [this repo](https://github.com/tttmmmyyyy/fixlang-registry).
You can add other registry files by specifying them in the [configuration file](#configuration-file).
To list all available projects registered in the registry files, use "fix deps list" command.

As mentioned above, the [[dependencies]] element specifies a range of versions, not a specific version, for each dependency.
Specific version (commit) to use for each dependency is written in the "fixdeps.lock" file.
This file is automatically generated when you run the "fix deps add" command, and you can update it to use a newer version by running the "fix deps update" command.

The "fix deps install" command installs the dependencies written in the "fixdeps.lock" file into the ".fix" directory.
This command is automatically called from "fix build" or "fix run" command.

## Configuration file

You can specify the bahavior of "fix" command by a configuration file named ".fixconfig.toml" in the home directory.

The fields allowed in the configuration file are as follows:

```
# URLs / paths to the registry files.
# "fix deps add {proj-name}@{ver-req}" command will search the project in the registry files from the first to the last, and if found, adds "[[dependencies]]" section to the project file at the current directory.
# The default registry "https://raw.githubusercontent.com/tttmmmyyyy/fixlang-registry/refs/heads/main/registry.toml" is implicitly added to the end of the list.
registries = [
    "https://first-searched-registry.com/registry.toml",
    "https://second-searched-registry.com/registry.toml",
    "/path/to/my_registry.toml"
]
```

## Generating documentation

`fix docs` subcommand generates documentations (markdown files) for a Fix project.
This command requires the project file to be present in the current directory.

Consecutive line comments above declarations are recognized as documentations:

```
// This is a documentation comment for the module.
module Main;

// This is a documentation comment for a value.
truth : I64;
truth = 42;

// This is a documentation comment for a type.
type MyType = struct { x : I64 };

// This is a documentation comment for a trait.
trait a : MyTrait {
    // This is a documentation comment for a trait method.
    to_number : a -> I64;
}

// This is a documentation comment for a trait implementation.
impl MyType : MyTrait  {
    to_number = |mt| mt.@x;
}
```

## Language Server Protocol

Running `fix language-server` starts a language server which supports Language Server Protocol (LSP). 
Language client extension for VSCode is available in [here](https://marketplace.visualstudio.com/items?itemName=tttmmmyyyy.fixlang-language-client).

The language server requires [the project file](#project-file) to recognize the Fix source files.

Each time you save a file, the language server will attempt to diagnose the Fix program.
The information obtained in the latest successful diagnostics is used to comletion, hover or go-to-definition, etc.
So to update the information, you need to write correct Fix code and save the file. 
[`Std::undefined`](/std_doc/Std.md#undefined-----a) will be useful to do so.

### Specifying parameter list in the documentation comment as a hint to the language server

The language server can provide better features if it knows the parameter list of a function.
For example, when you complete the function name `foo` which has parameters `x` and `y`, it can insert placeholder arguments like `foo(x, y)`.

However, since Fix is a functional programming language, it is ambiguous what the parameter list of a function is, as shown in the following example:

```
foo : I64 -> I64 -> I64 -> I64;
foo = |x, y| (
    if x == 1 {
        |z| x + y + z
    } else {
        |k| (x + y) * k
    }
);
```

The parameter list of this function is `x`, `y`, `z` or `x`, `y`, `k`?

To address this, you can specify the parameter list in the "Parameters" section of the documentation comment of a function.
To do this, write as follows:

```
// # Parameters
// * `x` - the first argument
// * `y` - the second argument
foo : I64 -> I64 -> I64 -> I64;
```

This comment indicates that `foo` is a function with two arguments `x` and `y` in typical cases.
Then, when you complete the function name `foo`, the language server will insert a text `foo(x, y)`.
If `foo` is completed after a dot, e.g., `y.foo`, it will be inserted as `y.foo(x)`.

Here, we explain the specification of the documentation comment in more detail.

- The language server interprets the documentation comment as a Markdown, and searches the "Parameters" section of level 1 or 2.
- If found, it extracts parameter names from all lists, i.e., lines starting with `* ` or `- `.
- The parameter names should be enclosed in backquotes ("`").
- You can contain type annotations in the backquotes, e.g., `x : I64`, which will be ignored by the language server.

## Debugging Fix program

Running `fix build`, `fix run` or `fix test` with `-g` option generates executable binary with DWARF debugging information. 
Then you can debug the binary by lldb, gdb or other GUI debuggers such as [CodeLLDB](https://marketplace.visualstudio.com/items?itemName=vadimcn.vscode-lldb). 

In VSCode, you cannot put a breakpoint in *.fix files by default. As a workaround, open "Preferences" and turn "Allow Breakpoints Everywhere" ON.

Moreover, if you add `--backtrace` option to `fix build`, `fix run` or `fix test`, a stack trace will be printed when a panic occurs. If you use it with `-g` option, function names and line numbers will be shown in the stack trace.

Other notes on debugging Fix program:
- Unlike other languages, Fix does not release local variables at the end of their scope, but at the last point of use. So if you break after the last use of a local variable, the debugger may show an invalid value.
- Currently, we are not able to tell the debugger the size of an array which is determined at run time. So we are always setting the array size to 100 in the debug information. You cannot show elements indexed after 100, and if the array is shorter than 100, invalid values are shown.

# Other documents

*[Document for all modules in the default registry](https://tttmmmyyyy.github.io/fixlang-docpage-generator/)