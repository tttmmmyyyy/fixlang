Fix-lang
====

Overview

Fix is *planned to be* a functional language of Haskell-like syntax and type systems, with the following features:
- Eager evaluation
- Reference counting garbage collection, with no cycles are made!
    - Self-referencing expression such as "let f = g f in f" is converted to "fix g".
- Mutability by reference counter
    - For example, the update in "update array idx (+1)" doesn't copy array when reference counter of array is one.

## Description

## Demo

## Requirement

## Usage

## Install

## Contribution

## Licence

## Author

## ToDo:

* Refactoring:
    * system_functions -> runtimes
    * make call of SystemFunctions more easily
    * move builder functions to method of GenerationContext
    * make push_builder better
    * replace pointer cast to to_ptr_type