Documentation
===

# Syntax

## Module declaration

Each source file needs a module declaration at the first.

```
module Main;
```

Module name is used as the namespace of global names, types and traits defined in the source file.

## Global values and `main`

You can define a global value and it's type and name as follows.

```
truth: Int;
truth = 42;
```

The name of value has to start with a lower-case alphabet.

When fix program starts to run, the runtime calculates for a `main` global value of type `Int` and prints its value.

## Let binding

To define a local name and it's value, use `let`-binding. 

```
module Main;

main : Int;
main = let x = 5 in 2 + x;
```

```
module Main;
main : Int;
main = (
    let x = 3;
    let y = 5;
    x + y
);
```
The syntax is `let {name} = {expression_0} in {expression_1}` or `let {name} = {expression_0}; {expression_1}`.

If the name of the lhs of `let`-binding is already in the scope, `let` evaluates `{expression_0}` in the old scope (i.e., with the old value of the name) and evaluates `{expression_1}` in the new scope (i.e., with the new value of the name).

Fix's `let`-binding doen't allow to make recursive definition. To define a recursive function locally, use `fix` built-in function.

## If

The syntax of `if` is `if {condition} then {expression_0} else {expression_1}`. The type of `{condition}` has to be `Bool`, and The types of `{expression_0}` and `{expression_1}` must coincide. Boolean value literals are `True` and `False`.

```
if False then 1 else 0 // evaluates to 0
```

## Function application

To apply a function `f` to a variable `x`, just write `f x`. This is left-associative: `f x y` is interpreted as `(f x) y`.

```
neg 3 // -3 -- `neg` is a built-in function that takes a Int value and returns negative of it.
```

## Function definition (Lambda abstraction)

You can make a function value (which is similar to things called "lambda" or "closure" in other languages) by `\{name} -> {expression}`. 

```
let x = 3;
let add_x = \n -> n + x;
add_x 4 + add_x 5 // (4 + 3) + (5 + 3) = 15
```

## Recursion

Recursion in definition of global value is allowed.

```
module Main;

fib : Int -> Int;
fib = \n -> (
    if n == 0 then
        0
    else if n == 1 then
        1
    else
        fib (n-1) + fib (n-2)
);

main : Int;
main = fib 30; // 832040
```

On the other hand, Fix's `let`-binding doen't allow to make recursive definition. To define a recursive function locally, use `fix` built-in function.

## Type annotation

## Types

### Arrow types

### Tuples

### Struct definition

### Union definition

### Type parameters

## Traits

### Trait bound

## Higher-kinded types

# Built-in features

## Types

## Traits

## Functions

## Operators