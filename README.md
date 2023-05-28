Fix-lang
====

## Overview

Fix is a programming language with the following features: 
- Functional: All functions have no side effect and all values are immutable. This reduces bugs caused by state management failures.
- O(1) update of arrays and structures: Despite the 1st feature, Fix mutates a value if the mutation cannot be observed. For example, `let array1 = array0.set(10, 42);` defines a new array `array1` that is almost identical to `array0` but with the 10th element replaced by 42. If `array0` will not be referenced later, Fix will update the 10th element of `array0` and rename it as `array1`. On the other hand, if `array0` may be used later, Fix creates `array1` by cloning `array0` and setting the 10th element to 42, keeping immutability.
- Familier syntax: The syntax of Fix is more similar to languages such as C++ or Rust than to other functional languages such as Haskell. Even if you have never learned a functional language, you will be able to learn Fix quickly.

In another perspective, Fix is a language which uses reference counting to provide garbage collection and interior mutability. To avoid circular reference, all values are semantically immutable and it restricts dynamic recursive definition and forces to use fixed-point combinator instead. To reduce copy cost on "modify" operation of a value, Fix mutates it if the reference counter is one.

You can try Fix in [Google Colaboratory](https://colab.research.google.com/github/tttmmmyyyy/fixlang/blob/main/run_fix_2.ipynb).

(This project is still a WIP and has no practical use yet.)

## Install (macOS / WSL)

- Install [Rust](https://www.rust-lang.org/tools/install).
- Install llvm12.0.1. It is recommended to use [llvmemv](https://crates.io/crates/llvmenv).
    - In macOS, llvmenv installs llvm to "~/Library/Application Support/llvmenv/12.0.1", but llvm-sys currently doesn't understand path with a whitespace correctly, so you need to copy/move "12.0.1" directory to another path.
- Set LLVM_SYS_120_PREFIX variable to the directory to which llvm installed.
- `git clone https://github.com/tttmmmyyyy/fixlang.git && cd fixlang`.
- `cargo install --path .`. Then the compiler command `fix` will be installed to `~/.cargo/bin`.

## Usage

- You can run the source file (with extension ".fix") by `fix run -f {source-file}`.
- If you want to build executable binary, run `fix build -f {source-file}.`.

## Tutorial / references

See [document](/Document.md).