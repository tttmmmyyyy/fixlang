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
    * move builder functions to method of GenerationContext
    * make push_builder better
    * replace pointer cast to to_ptr_type
    * place ObjectType::lam_obj_type().to_struct_type(gc.context) to global (like other types)
        * int_obj_type and bool_obj_type
        * take gc instead of context.
    * move literal generator to somewhere
    * use push_builder in generate_lam
    * remove ExprCode
    * naming convention of methods of GeneratorContext.
    * start_function method
    * RAII of lock_used_later