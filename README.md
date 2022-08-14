Fix-lang
====

Overview

Fix is *planned to be* a functional language of Haskell-like syntax and type systems, with the following features:
- Eager evaluation
- Reference counting garbage collection, with no cycles guaranteed to be made.
-- Self-referencing expression such as "let f = g f in f" is converted to "fix g".
- Mutability using reference counter.
-- For example, the "update" in "update array idx (+1)" doesn't copy array when reference counter of a is one.

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