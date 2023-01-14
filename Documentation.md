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

Fix's `let`-binding doesn't allow to make recursive definition. To define a recursive function locally, use `fix` built-in function.

## If

The syntax of `if` is `if {condition} then {expression_0} else {expression_1}`. The type of `{condition}` has to be `Bool`, and The types of `{expression_0}` and `{expression_1}` must coincide. Boolean value literals are `true` and `false`.

```
if false then 1 else 0 // evaluates to 0
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

You can make recursive global function as usual.

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

On the other hand, Fix's `let`-binding doesn't allow to make recursive definition. To define a recursive function locally, use `fix` built-in function.

## Type annotation

## Types

### Boxed and unboxed types

Types in fix are divided into boxed types and unboxed types. 

* Value of boxed types are allocated in heap memory. Local names and struct / union fields whose types are boxed are compiled as pointers to the values. 
* Values of unboxed types are directly embedded into the stack memoroy, structs and unions. 

### Arrow types

Arrow types are boxed (in principle).

### Tuples

Tuple types are unboxed.

### Structs

You can define a new struct by `type {type_name} = struct ({field_name}: {field_type},...);`. The `{type_name}` must start with a uppercase alphabet. 

Example:
```
module Main;

type Product = struct (price: Int, sold: Bool);
```

For each struct, the following methods are defined in the namespace of {type_name} automatically: 
- `new : {field_type}... -> {type_name}`
    - For the `Product` example above, `Main.Product.new : Int -> Bool -> Product`.
- `get_{field_name} : {type_name} -> {field_type}`
    - For the `Product` example above, `Main.Product.get_price : Product -> Int` and `Main.Product.get_sold : Product -> Bool`.
- `mod_{field_name} : ({field_type} -> {field_type}) -> {type_name} -> {type_name}`
    - For the `Product` example above, `Main.Product.mod_price : (Int -> Int) -> Product -> Product` and `Main.Product.mod_sold : (Bool -> Bool) -> Product -> Product`. 
    - This function receives a transformer function on a field and extends it to the transformer of a struct value.
    - This function clones the given struct value if it is shared between multiple references.
- `mod_{field_name}! : ({field_type} -> {field_type}) -> {type_name} -> {type_name}`
    - For the `Product` example above, `Main.Product.mod_price! : (Int -> Int) -> Product -> Product` and `Main.Product.mod_sold! : (Bool -> Bool) -> Product -> Product`. 
    - This function always update the given struct value. If the given struct value is shared between multiple references, this function panics (i.e., stops the execution of the program).

Convenient `set_{field_name}` and `set_{field_name}!` functions (or more is general lens function) will be added in the future.

Struct types are boxed by default.

### Unions

You can define a new union by `type {type_name} = union ({field_name}: {field_type},...);`. The `{type_name}` must start with a uppercase alphabet. 

Example:
```
module Main;

type Weight = union (pound: Int, kilograms: Int);
```

For each struct, the following methods are defined in the namespace of {type_name} automatically: 
- `{field_name} : {field_type} -> {type_name}`
    - For the `Weight` example above, `Main.Weight.pound : Int -> Weight` and `Main.Weight.kilograms : Int -> Weight`.
- `as_{field_name} : {type_name} -> {field_type}`
    - For the `Weight` example above, `Main.Weight.as_pound : Weight -> Int` and `Main.Weight.as_kilograms : Weight -> Int`.
    - If the given union value doesn't carry `{field_name}`, this function panics.
- `is_{field_name} : {type_name} -> Bool`
    - For the `Weight` example above, `Main.Weight.is_pound : Weight -> Bool` and `Main.Weight.is_kilograms : Weight -> Bool`.

### Type parameters

## Traits

### Trait bound

## Higher-kinded types

# Built-in features

## Types

### Std.Array

`Std.Array` is the type of fixed-length array.

- `Std.Array.new : Int -> a -> Std.Array a`
    - Creates an array of the specified length and elements of the specified value.
- `Std.Array.from_map : Int -> (Int -> a) -> Std.Array a`
    - Creates an array of the specified length and elements specified by the function given as the second argument at each index.
- `Std.Array.get : Int -> Std.Array a -> a`
    - Returns an element of an array at the specified index.
- `Std.Array.set : Int -> a -> Std.Array a -> Std.Array a`
    - Updates a value of an element at the specified index of an array.
    - This function clones the given array if it is shared between multiple references.
- `Std.Array.set! : Int -> a -> Std.Array a -> Std.Array a`
    - Updates a value of an element at the specified index of an array.
    - This function always update the given array. If the given array is shared between multiple references, this function panics.
- `Std.Array.len : Std.Array a -> Int`
    - Returns the length of an array.

## Functions

### Std.fix : ((a -> b) -> a -> b) -> a -> b

`fix` enables you to make a recursive function locally. The idiom is: `fix \loop -> \var -> (expression calls loop)`.

```
module Main;

main : Int;
main = (
    let fact = fix \loop -> \n -> if n == 0 then 1 else n * loop (n-1);
    fact 5 // evaluates to 5 * 4 * 3 * 2 * 1 = 120
);
```

### Std.loop : s -> (s -> Std.LoopResult s r) -> r

`loop` enables you to make a loop. `LoopResult` is a union type defined as follows: 

```
type LoopResult s r = union (s: continue, r: break);
```

`loop` takes two arguments: the initial state of the loop `s0` and the loop body function `body`. It first calls `body` on `s0`. If `body` returns `break r`, then the loop ends and returns `r` as the result. If `body` returns `continue s`, then the loop calls again `body` on `s`.

```
module Main;
    
main : Int;
main = (
    loop (0, 0) \state -> 
        let i = state.get_0;
        let sum = state.get_1;
        if i == 100 then 
            break sum 
        else
            continue (i+1, sum+i)
); // evaluates to 0 + 1 + ... + 99 
```

## Operators

The following is the table of operators sorted by it's precedence (operator of higher precedence appears earlier).

| Operator       | Associativity | Trant / method                     | Explanation                                                 | 
| -------------- | ------------- | ---------------------------------- | ----------------------------------------------------------- | 
| (whitespace)   | left          | -                                  | function application                                        | 
| .              | left          | -                                  | right-to-left function application: x.f = f x               | 
| - (minus sign) | -             | Std.Neg / neg                      | negative of number                                          | 
| !              | -             | Std.Not / not                      | logical NOT                                                 | 
| *              | left          | Std.Mul / mul                      | multiplication of numbers                                   | 
| /              | left          | Std.Div / div                      | division of numbers                                         | 
| %              | left          | Std.Rem / rem                      | reminder of division                                        | 
| +              | left          | Std.Add / add                      | addition of numbers                                         | 
| - (minus sign) | left          | Std.Sub / sub                      | subtraction of numbers                                      | 
| ==             | left          | Std.Eq / eq                        | equality comparison                                         | 
| !=             | left          | -                                  | `x != y` is interpreted as `!(x == y)`                      | 
| <=             | left          | Std.LessThanOrEq / less_than_or_eq | less-than-or-equal-to comparison                            | 
| >=             | left          | -                                  | `x >= y` is interpreted as `y <= x`                         | 
| <              | left          | Std.LessThan / less_than           | less-than comparison                                        | 
| >              | left          | -                                  | `x > y` is interpreted as `y < x`                           | 
| &&             | left          | Std.And / and                      | logical AND                                                 | 
| &#124;&#124;   | left          | Std.Or / or                        | logical OR                                                  | 
| $              | right         | -                                  | right associative function application: f $ g $ x = f (g x) | 

# Features of "fix" command