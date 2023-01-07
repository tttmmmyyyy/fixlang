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

main : Int;
main = (
    let arr = Array.new 31 0;
    let arr = arr.set! 0 0;
    let arr = arr.set! 1 1;
    let arr = loop (2, arr) \state -> (
        let idx = state.get_0;
        let arr = state.get_1;        
        if idx == arr.len then 
            break arr
        else
            let x = arr.get (idx-1);
            let y = arr.get (idx-2);
            let arr = arr.set! idx (x+y);
            continue $ (idx+1, arr)
    );
    arr.get 30 // 832040
);
```
Currently, what fix can do is only to calculate and print a single `Int` value! In a source file, you need to declare the module name for the source file as `Main` and define `main` symbol of type `Int`.

`Array.new` in the first line of `main` is the constructor function for `Array`. It takes the length and the initial value and returns a new Array. The `Array` here is not a type, but a namespace: the name of the constructor function is `new`, and it is defined in the namespace `Std.Array`, where `Std` is a namspace for standard libraries / built-in functions. So the full-name of the constructor function is `Std.Array.new`, but you can omit some prefix of the full namespace when the compiler can infer it.

To apply a function `f` to a variable `x`, just write `f x`. This is left-associative: `f x y` is interpreted as `(f x) y`. To define a function that takes the variable `arg` as the argument, write `\arg -> {an expression that may use arg}`. All functions in fix takes a single argument, and a standard way to define a function that takes multiple arguments is to take the first argument and returns a function that handles the second argument. Another way is to define a function that takes a tuple.

The operator `.` in the `arr.set! 0 0` and `arr.get 30` is NOT the composition operator (as used in Haskell), but the right-to-left application operator: `x.f = f x`. The precedence between two ways of application is whitespace (usual application) > `.` . This allows you to write `obj.method arg` to call a function `method: Arg -> Obj -> Result` on `obj` as if you are writing OOP languages. There is also right-associative `$` operator that is well-known in Haskell and useful for reducing parentheses: `f $ x = f x` and `f $ g $ x = f (g x)`. The precedence of `$` is weaker than that of whitespace (usual application) and `.`.

`set!: Int -> a -> Array a -> Array a` in the code above is a method of Array which updates the value of the given array if it is uniquely referenced (i.e. the reference counter is one) or stops the program otherwise. This allows you to avoid cloning array while keeping purity (no side effect). If you are ok for cloning array when it is shared between multiple references, use `set` method instead.

The `loop` function takes two arguments: the initial state of the loop `s0` and the loop body function `body`. It first calls `body` on `s0`. The return value of `body` has to be made by `break` function or `continue` function. If `body` returns `break r`, then the loop ends and returns `r` as the result. If `body` returns `continue s`, then the loop calls again `body` on `s`.

In the above exapmle, the state of the loop is `(Int, Array Int)`, which is the tuple (pair) of the loop index and the array under construction. It starts from index 2, and the body stores the next value of fibonacci sequence to the array, or breaks when the loop index reached to the length of the array.

For more informations on syntax and built-in functions, see [documentation](/Documentation.md).