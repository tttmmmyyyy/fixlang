Documentation
===

# Syntax

## Module definition

Each source file needs a module definition at the first.

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

`Main` module has to include a `main` global value of type `IOState -> ((), IOState)`. When fix program starts to run, the runtime generates an `IOState` value and pass it to `Main.main` function.

## Let binding

To define a local name and it's value, use `let`-binding. 

```
let x = 5 in 2 + x // 7
```

```
let x = 3;
let y = 5;
x + y // 8
```
The syntax is `let {name} = {expression_0} in {expression_1}` or `let {name} = {expression_0}; {expression_1}`.

If the name of the lhs of `let`-binding is already in the scope, `let` evaluates `{expression_0}` in the old scope (i.e., with the old value of the name) and evaluates `{expression_1}` in the new scope (i.e., with the new value of the name).

Fix's `let`-binding doesn't allow making recursive definition. To define a recursive function locally, use `fix` built-in function.

## If

The syntax of `if` is `if {condition} then {expression_0} else {expression_1}`. The type of `{condition}` has to be `Bool`, and The types of `{expression_0}` and `{expression_1}` must coincide. Boolean value literals are `true` and `false`.

```
if false then 1 else 0 // evaluates to 0
```

## Function application

To apply a function `f` to a variable `x`, just write `f(x)`. `f(x, y)` is interpreted as `(f(x))(y)`.

```
neg(3) // -3 -- `neg` is a built-in function that takes a Int value and returns negative of it.
```

## Function definition (Lambda abstraction)

You can make a function value (which is similar to things called "lambda" or "closure" in other languages) by `|arg| body`. `|arg0, arg1| body` is intepreted as `|arg0| (|arg1| body)`.

```
let x = 3;
let add_x = |n| n + x;
add_x(4) + add_x(5) // (4 + 3) + (5 + 3) = 15
```

## Recursion

You can make recursive global function as usual.

```
module Main;

fib : Int -> Int;
fib = |n| (
    if n == 0 then
        0
    else if n == 1 then
        1
    else
        fib(n-1) + fib(n-2)
);

main : IOState -> ((), IOState);
main = print $ fib(30).to_string; // 832040
```

On the other hand, Fix's `let`-binding doesn't allow to make recursive definition. To define a recursive function locally, use `fix` built-in function.

## Type annotation

## Basic types

### Boxed and unboxed types

Types in fix are divided into boxed types and unboxed types. Boxed types and unboxed types are similar to things called as "reference types" and "value types" in other languages, respectively.

* Value of boxed types are allocated in heap memory. Local names and struct / union fields whose types are boxed are compiled as pointers to the values. 
* Values of unboxed types are directly embedded into the stack memory, structs and unions. 

In general, types that contain a lot of data (such as `Array`) are suited to be boxed because boxed types have lower copying costs. On the other hand, types containing small data (such as `Int`) can be unboxed to reduce the cost of increasing or decreasing the reference counter.

### Functions

Types of functions are represented as `a -> b`. For example, `Int -> Bool` is the type of functions which takes an `Int` value and returns a `Bool` value.

The type constructor `->` is right-associative: `a -> b -> c` is interpreted as `a -> (b -> c)`.

Functions are boxed, because it may contain many captured values.

### Tuples

Tuple types are unboxed, because tuple is intended to have only a few fields. If you want to use many fields, you should define a new struct.
Tuples are special forms of [structs](#Structs) whose field names are `0`, `1`, `2`, etc. 

### Unit

Unit `()` is a type allows only one value, which is also written as `()`.

### Array

`Std.Array` is the type of fixed-length array. This is a basic type in fix and used to construct `Std.Vector`, the type of variable-length array. `Std.Array` is a boxed type.

### Structs

You can define a new struct by `type {type_name} = struct ({field_name}: {field_type},...);`. The `{type_name}` must start with a uppercase alphabet. 

Example:
```
module Main;

type Product = struct { price: Int, sold: Bool };
```

You can construct a struct value by the syntax `{struct_name} { ({field_name}: {field_value}) } `:

```
let product = Product { price: 100, sold: false };
```

For each struct, the following methods are defined in the namespace of {type_name} automatically: 
- `new : {field_type}... -> {struct_type}`
    - Construct a struct value.
    - For the `Product` example above, `Main.Product.new : Int -> Bool -> Product`.
- `@{field_name} : {struct_type} -> {field_type}`
    - Get the field value of a struct value.
    - For the `Product.price` example above, `Main.Product.@price : Product -> Int`.
- `={field_name} : {struct_type} -> {field_type} -> {field_type}`
    - Set the field value of a struct value.
    - This function clones the struct value if it is shared between multiple references.
    - For the `Product.price` example above, `Main.Product.=price : Int -> Product -> Product`.
- `={field_name}! : {struct_type} -> {field_type} -> {field_type}`
    - Set the field value of a struct value.
    - This function always updates the struct value. If the struct value is shared between multiple references, this function panics.
    - For the `Product.price` example above, `Main.Product.=price! : Int -> Product -> Product`.
- `mod_{field_name} : ({field_type} -> {field_type}) -> {struct_type} -> {struct_type}`
    - Modify the field value of a struct value by a function which acts to a field value.
    - For the `Product.price` example above, `Main.Product.mod_price : (Int -> Int) -> Product -> Product`.
    - This function clones the struct value if it is shared between multiple references.
- `mod_{field_name}! : ({field_type} -> {field_type}) -> {struct_type} -> {struct_type}`
    - Modify the field value of a struct value by a function which acts to a field value.
    - This function always updates the struct value. If the struct value is shared between multiple references, this function panics.
    - For the `Product.price` example above, `Main.Product.mod_price! : (Int -> Int) -> Product -> Product`. 

Structs are boxed by default because they are assumed to have many fields. To define unboxed struct type, write `unbox` specifier before `struct`.

```
type Product = unbox struct (price: Int, sold: Bool);
```

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

Unions are unboxed by default because they only contains a single value at a time. To define boxed union type, write `box` specifier before `struct`.

```
type Weight = box union (pound: Int, kilograms: Int);
```

### Type parameters

## Traits

### Trait bound

## Higher-kinded types

## Namespaces

Namespaces of global names (i.e., names of global values, types and traits) can be specified using the following syntax: `namespace {namespace} { ... }`.

For example, in the following program,

```
module Main;

namespace TheNameSpace {
    truth : Int;
    truth = 42;
}

truth : Bool;
truth = true;
```

two global values are defined: `Main.TheNameSpace.truth : Int` and `Main.truth : Bool`.

# Built-in / library features

## Types

### Std.Int

`Std.Int` is the type of 64-bit signed integers.

### Std.Bool

`Std.Bool` is the type of boolean values. 

### Std.Byte

`Std.Byte` is the type of 8-bit unsigned integers.

### Std.Array

`Std.Array` is the type of fixed-length arrays.

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

### Std.Vector

`Std.Vector` is the type of variable-length array.

```
type Vector a = unbox struct ( len : Int, data : Array a );
```

### Std.String

`Std.String` is the type of strings.

```
type String = unbox struct ( data : Vector Byte );
```

## Functions

### Std.fix : ((a -> b) -> a -> b) -> a -> b

`fix` enables you to make a recursive function locally. The idiom is: `fix $ |loop, var| -> (expression calls loop)`.

```
module Main;

main : IOState -> ((), IOState);
main = (
    let fact = fix $ |loop, n| if n == 0 then 1 else n * loop (n-1);
    print $ fact(5).to_string // evaluates to 5 * 4 * 3 * 2 * 1 = 120
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
    
main : IOState -> ((), IOState);
main = (
    let sum = (
        loop((0, 0), |(i, sum)|
            if i == 100 then 
                break $ sum 
            else
                continue $ (i+1, sum+i)
        )
    );
    print $ sum.to_string
); // evaluates to 0 + 1 + ... + 99 
```

### Std.print : Std.String -> Std.IOState -> ((), Std.IOState)

### Std.Debug.debug_print : Std.String -> ()

### Std.Debug.abort : () -> a

### Std.Debug.assert_eq : [a: Eq] String -> a -> a -> ()

## Traits

### Std.ToString

- `to_string : [a: Std.ToString] a -> Std.String`

## Operators

The following is the table of operators sorted by it's precedence (operator of higher precedence appears earlier).

| Operator       | Associativity | Trait / method                     | Explanation                                                 | 
| -------------- | ------------- | ---------------------------------- | ----------------------------------------------------------- | 
| f(x)           | left          | -                                  | function application                                        | 
| .              | left          | -                                  | right-to-left function application: x.f = f(x)              | 
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
| $              | right         | -                                  | right associative function application: f $ g $ x = f(g(x)) | 

# Features of "fix" command