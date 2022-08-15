Fix-lang
====

## Overview

Fix is *planned to be* a functional language of Haskell-like syntax and type systems, with the following features:
- Eager evaluation
- Reference counting garbage collection, where it is guaranteed that no cycles will be created.
    - Self-referencing expression such as "let f = g f in f" is not permitted and should be written as "fix g".
- Updating uniquely owned object
    - For example, the evaluation of "update array idx (+1)" doesn't copy array when reference counter of array is one.

## Description

## Demo

## Requirement

## Usage

## Install

## Contribution

## Licence

## Author

## ToDo:

* Type checking
* Add Array
