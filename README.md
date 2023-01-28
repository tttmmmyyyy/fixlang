Fix-lang
====

## Overview

Fix is a functional language focused on ease of learning and use.

(This project is still a WIP and has no practical use yet.)

## Install (macOS / WSL)

- Install [Rust](https://www.rust-lang.org/tools/install).
- Install llvm12.0.1. It is recommended to use [llvmemv](https://crates.io/crates/llvmenv).
    - In macOS, llvmenv installs llvm to "~/Library/Application Support/llvmenv/12.0.1", but llvm-sys currently doesn't understand path with a whitespace correctly, so you need to copy/move "12.0.1" directory to another path.
- Set LLVM_SYS_120_PREFIX variable to the directory to which llvm installed.
- `git clone https://github.com/tttmmmyyyy/fixlang.git && cd fixlang`.
- `cargo build --release`. Then the compiler binary will be generated at "fixlang/target/release/fix". 
- You can run the source file of fix-lang by `fix run source-file.fix`, or build object file `source-file.o` by `fix build source-file.fix`. In the latter case, by passing the object file to gcc (`gcc source-file.o`) will generate executable binary.

## Explanation / basic example

You can learn the syntax by reading files in the "examples" directory (or codes in tests.rs). The following is a basic example ("exapmles/fib_loop.fix") about which I explain below.

```
module Main;

main : IOState -> ((), IOState);
main = (
    let arr = Array.new(31, 0);
    let arr = arr.set!(0, 0);
    let arr = arr.set!(1, 1);
    let arr = loop((2, arr), |(idx, arr)|
        if idx == arr.len then 
            break $ arr
        else
            let x = arr.get(idx-1);
            let y = arr.get(idx-2);
            let arr = arr.set!(idx, x+y);
            continue $ (idx+1, arr)
    );
    print $ arr.get(30).to_string // 832040
);
```
In a source file, you need to declare the module name for the source file as `Main` and define `main` object of type `IOState -> ((), IOState)`. The runtime generates a value of `IOState` and pass it to `Main.main` function.

`Array.new` in the first line of `main` is the constructor function for `Array`. It takes the length and the initial value and returns a new Array. The `Array` here is not a type, but a namespace: the name of the constructor function is `new`, and it is defined in the namespace `Std.Array`, where `Std` is a namspace for standard libraries / built-in functions. So the full-name of the constructor function is `Std.Array.new`, but you can omit some prefix of the full namespace when the compiler can infer it.

Semantically, all functions in Fix has a single parameter. To create a function which has a parameter `param` and returns the value of an expression `body`, write `|param| body`. You can also write `|param0, param1| body` (or more if necessary), which is just interpreted as `|param0| (|param1| body)` i.e., a function that takes `param0` and returns an function `|param1| body`. Such a function can be roughly called a "two-parameter function". The type of (N+1)-parameter function is written as `Param0 -> Param1 -> ... -> ParamN -> Result`.

Note that the expression `|(fst, snd)| body` represents a single-parameter function which takes a pair `(fst, snd)`, not a two-parameter function.

To call a function `f` on an argument `x`, write `f(x)`. You can call a two-parameter function `f` on arguments `x` and `y` by `f(x, y)`, which is a syntax sugar of `(f(x))(y)`. There is also a function application operator `$` which is well-known in Haskell and useful for reducing parentheses: `f $ x = f(x)` and `f $ g $ x = f(g(x))`.

The operator `.` in `arr.set!(0, 0)` and `arr.get(30)` is NOT the composition operator (as in Haskell), but the right-to-left application operator: `x.f = f(x)`. The precedence between three ways of function application is `f(x)` > `x.f` > `f $ x`. For example, for a function `method` of type `Param -> Obj -> Result`, `obj.method(arg)` is interpreted as `obj.(method(arg)) = method(arg, obj)`, not as `(obj.method)(arg)`. We sometimes call a function of type `Param0 -> ... -> ParamN -> Obj -> Result` as a "method" on the type `Obj` that has N+1 parameters and returns a value of type `Result`. A method can be called by `obj.method(arg0,...,argN)` as if you are writing OOP languages, due to the above-mentioned precedence.

`set!: Int -> a -> Array a -> Array a` in the program above is a method of Array which updates the value of the given array if it is uniquely referenced (i.e. the reference counter is one) or stops the program otherwise. This allows you to avoid cloning array while keeping purity (no side effect). If you are ok for cloning array when it is shared by multiple references, use `set` method instead.

The `loop` function takes two arguments: the initial state of the loop `s0` and the loop body function `body`. It first calls `body` on `s0`. The return value of `body` has to be made by `break` function or `continue` function. If `body` returns `break(r)`, then the loop ends and returns `r` as the result. If `body` returns `continue(s)`, then the loop calls again `body` on `s`.

In the above example, the state of the loop is `(Int, Array Int)`, which is the tuple (pair) of the loop counter and the array under construction. It starts from counter 2, and the body stores the next value of fibonacci sequence to the array, or breaks when the loop counter reached to the length of the array.

In the last, this prgram prints the the 30th element of the fibonacci array. The `print` function has type `print: String -> IOState -> ((), IOState)`, so `print some_string` matches to the type of `main`. The `IOState` type is a virtual type that represents the outer state of the Fix program, and the `print` function is considered to be a function that changes the outer state by printing a string to the screen. Since `print` updates the outer state, it only accepts a unique `IOState` value, as like `set!` for `Array`. If you duplicate an `IOState` value and pass it to `print`, then it stops the program.

For more informations on syntax and built-in functions, see [documentation](/Documentation.md).