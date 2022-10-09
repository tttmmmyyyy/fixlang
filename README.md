Fix-lang
====

## Overview

Fix is a simple and easy to learn/use programming language, with the following features:
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
    * Do boundary check in read_array and write_array.
* Comment