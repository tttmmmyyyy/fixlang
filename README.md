Fix-lang
====

## Overview

Fix is a functional language focused on ease of learning and use.

(This project is still a WIP and has no practical use yet.)

## Install

- Install llvm12.0.1 by using [llvmemv](https://crates.io/crates/llvmenv).
    - In macOS, llvmenv installs llvm to "~/Library/Application Support/llvmenv/12.0.1", but llvm-sys currently doesn't understand path with a whitespace correctly, so you need to copy/move "12.0.1" directory to another path.
- Set LLVM_SYS_120_PREFIX variable to the directory to which llvm installed.
- Clone this repository and run example by "cargo run -- run ./example/array_and_fix.fix".

## Explanation / basic example

Currently no detailed tutorial / documentation is written, but you can know the syntax by reading files in the "examples" directory (or codes in tests.rs). The following is a basic example ("exapmles/array_and_fix.fix") about which I explain below.

```
module Main;

main : Int;
main = (
    let arr = Array.new 31 0;
    let arr = arr.set! 0 0;
    let arr = arr.set! 1 1;
    let calc_fib_array = (
        fix \loop -> \arr -> \n -> 
            if n == 31 
            then arr 
            else
                let x = arr.get (n-1);
                let y = arr.get (n-2);
                let arr = arr.set! n (x+y);
                loop arr (n+1)
    );
    let fib_array = calc_fib_array arr 2;
    fib_array.get 30 // 832040
);
```

The operator `.` in the code above is NOT the composition operator (as used in Haskell), but the right-to-left application operator: `x.f = f x`. The precedence between two ways of application is whitespace (usual application) > `.` . This allows you to write `obj.method arg` to call a function `method: Arg -> Obj -> Result` on `obj` as if you are writing OOP languages.

`set!: Int -> a -> Array a -> Array a` in the code above is a method of Array which updates the value of the given array if it is uniquely referenced (i.e. the reference counter is one) or stops the program otherwise. This allows you avoid cloning array and keeping purity (no side effect). If you are ok for cloning array when it is shared between multiple references, use `set` method instead.

In the `let` of Fix, you cannot make recursive binding. In the `let arr = arr.set! 0 0;` line in the code, `arr` in the right of equality refers to the name defined in the previous line. If you want to define recursive function locally, use `fix: ((a -> b) -> a -> b) -> a -> b` function. An idiom for making local recursive function by `fix` is: `let func = fix \loop -> \arg_of_func -> (body of func which calls loop);`. You can define global recursive function in an usual way.

Fix supports a way for making loop other than recursion. For this, see "loop.fix" example.