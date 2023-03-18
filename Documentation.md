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

To define a local name and it's value, use `let`-binding. The syntax is `let {name} = {expression_0} in {expression_1}` or `let {name} = {expression_0}; {expression_1}`.

For one-line expression, it is preferred to use `in`:
```
let x = 5 in 2 + x // 7
```

On the other hand, if you want to `{epxression_0}` and `{expression_1}` in other lines, it is better to use semicolon:
```
let x = 3;
let y = 5;
x + y // 8
```

If `{expression_0}` ranges several lines, it is good to put parentheses around `{expression_0}`:
```
let n_mod_2 = (
    if n % 2 == 0 {
        1
    } else {
        0
    }
);
...
```

If the name of the lhs of `let`-binding is already in the scope, `let` evaluates `{expression_0}` in the old scope (i.e., with the old value of the name) and evaluates `{expression_1}` in the new scope (i.e., with the new value of the name).

Fix's `let`-binding doesn't allow making recursive definition. To define a recursive function locally, use `fix` built-in function.

## If

The syntax of `if` is the following:
- `if cond { expr_0 } (else|;) { expr_1 }` where curly braces around `expr_1` is optional.
The type of `cond` has to be `Bool`, and The types of `expr_0` and `expr_1` must coincide.

For usual case, use `if cond { expr_0 } else { expr_1 }`:
```
if cond { 
    "cond is true!"
} else {
    "cond is false!"
}
```

To write "early return" pattern without introducing indent, it is good to omit curly braces around else-expression:
```
if edge_case { "a trivial value" };
"a complicated calculation"
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
main = print! $ fib(30).to_string; // 832040
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

`Std.Array` is the type of variable-length array. `Std.Array` is a boxed type.

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

NOTE: In a future, we will add lens functions such as `act_{field_name} : [f: Functor] ({field_type} -> f {field_type}) -> {struct_type} -> f {struct_type} `, which are generalization of `mod` functions.

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

### Std.Array

`Std.Array` is the type of variable-length arrays.

Methods:

- `__unsafe_set_length : Int -> Array a -> Array a`
    - Updates the length of an array, without uniqueness checking or validation of the given length value.
- `__unsafe_get : Int -> Array a -> a`
    - Gets a value from an array, without bounds checking and retaining the returned value.
- `__unsafe_set : Int -> a -> Array a -> Array a`
    - Sets a value into an array, without uniqueness checking, bounds checking and releasing the old value.
- `append : Array a -> Array a -> Array a`
    - Append an array to an array.
    - Note: Since `a1.append(a2)` puts `a2` after `a1`, `append(lhs, rhs)` puts `lhs` after `rhs`.    
- `force_unique : Array a -> Array a`
    - Force the uniqueness of an array.
    - If the given array is shared, this function returns the cloned array.
- `force_unique! : Array a -> Array a`
    - Force the uniqueness of an array.
    - If the given array is shared, this function panics.
- `from_map : Int -> (Int -> a) -> Array a`
    - Creates an array by a mapping function.
    - `from_map(n, f) = [f(0), f(1), f(2), ..., f(n-1)]`.
- `get : Int -> Array a -> a`
    - Returns an element of an array at an index.
- `get_length : Array a -> Int`
    - Returns the length of an array.
- `get_capacity : Array a -> Int`
    - Returns the capacity of an array.
- `make_empty : Int -> Array a`
    - Creates an empty array with specified capacity.
- `mod : Int -> (a -> a) -> Array a -> Array a`
    - Modifies a value of an element at the specified index of an array by a function.
    - This function clones the array if it is shared between multiple references.
- `mod! : Int -> (a -> a) -> Array a -> Array a`
    - This function clones the array if it is shared between multiple references.
    - This function always update the array. If the array is shared between multiple references, this function panics.  
- `new : Int -> a -> Array a`
    - Creates an array filled with the initial value.
    - The capacity is set to the same value as the length.
    - `new(n, x) = [x, x, x, ..., x]` (of length `n`).
- `pop_back : Array a -> Array a`
    - Pop an element at the back of an array.
    - If the array is empty, this function does nothing.
- `push_back : a -> Array a -> Array a`
    - Push an element to the back of an array.
- `reduce_length : Int -> Array a -> Array a`
    - Reduce the length of an array.
- `reserve : Int -> Array a -> Array a`
    - Reserves the memory region for an array.
- `set : Int -> a -> Array a -> Array a`
    - Updates a value of an element at an index of an array.
    - This function clones the given array if it is shared between multiple references.
- `set! : Int -> a -> Array a -> Array a`
    - Updates a value of an element at an index of an array.
    - This function always update the given array. If the given array is shared between multiple references, this function panics.
- `sort_by : ((a, a) -> Bool) -> Array a -> Array a`
    - Sort elements in a vector by "less than" comparator.
- `_sort_range_by_using_buffer : Array a -> Int -> Int -> ((a, a) -> Bool) -> Array a -> (Array a, Array a)`
    - Sort elements in a range of a vector by "less than" comparator.
    - This function receives a working buffer as the first argument to reduce memory allocation, and returns it as second element.

You can create array by the array literal syntax `[a0, a1, ..., an]`.

NOTE: In a future, we will add lens functions such as `act : [f: Functor] Int -> (a -> f a) -> Array a -> f (Array a)`, which are generalization of `mod` functions.

Implementing Traits:

- `[a : Eq] Array a : Eq`

### Std.Bool

`Std.Bool` is the type of boolean values, represented by 8-bit integer `1` (`true`) and `0` (`false`). 

### Std.Byte

`Std.Byte` is the type of 8-bit unsigned integers.

### Std.IOState

The virtual type that represents the state of world (=the outside of the Fix program). 

For example, `Std.IOState.print!(msg) : Std.IOState -> ((), Std.IOState)` function can be considered that it changes the state of the world by printing the message to the display. So it should receive `Std.IOState` and return the updated `Std.IOState` value paired with the result of the action (in this case, it is `()`, because printing message returns no result).

All functions that perform I/O action by `IOState` assert that the given state is unique.

Methods:

- `pure : () -> IOState -> ((), IOState)`
    - Makes a "do nothing" I/O action.
- `print! : String -> IOState -> ((), IOState)`
    - Prints a string to standard output.
- `println! : String -> IOState -> ((), IOState)`
    - Prints a string and a newline to standard output.

### Std.Int

`Std.Int` is the type of 64-bit signed integers.

Methods:

- `Std.Int._int_to_string : Int -> String`
    - Convert an integer to a decimal number string.
    - Implementation of trait method `Std.ToString.to_string`.

Implementing traits:

- `Std.ToString`

### Std.Iterator

Iterators (a.k.a. lazy lists) are generators of sequenced values.

Methods:

- `append : Iterator a -> Iterator a -> Iterator a`
    - Append an iterator to a iterator.
    - Note: Since `iter1.append(iter2)` puts `iter2` after `iter1`, `append(lhs, rhs)` puts `lhs` after `rhs`.    
- `count_up : Int -> Iterator Int`
    - Create an iterator that counts up from a number.
    - `count_up(n) = [n, n+1, n+2, ...]` (continues infinitely)
- `get_length : Iterator a -> Int`
    - Counts the length of an iterator.
- `intersperse : a -> Iterator a -> Iterator a`
    - Intersperse an elemnt between elements of an iterator.
    - Example: `Iterator.from_array([1,2,3]).intersperse(0) == Iterator.from_array([1,0,2,0,3])`
- `make_empty : Iterator a`
    - Creates an empty iterator.
- `filter : (a -> Bool) -> Iterator a -> Iterator a`
    - Filter elements by a condition function.
- `flatten : Iterator (Iterator a) -> Iterator a`
    - Flatten an iterator of iterators.
- `fold : b -> (b -> a -> b) -> Iterator a -> b`
    - Folds iterator from left.
    - `fold(init, op, [a0, a1, a2, ...]) = ...op(op(op(init, a0), a1), a2)...`
- `from_array : Array a -> Iterator a`
    - Create iterator from an array.
- `from_map : (Int -> a) -> Iterator a`
    - Create iterator from mapping function.
    - `from_map(f) = [f(0), f(1), f(2), ...]`
- `map : map : (a -> b) -> Iterator a -> Iterator b`
    - Apply a function to each value of iterator.
    - `map(f, [a0, a1, a2, ...]) = [f(a0), f(a1), f(a2), ...]`
- `next : Iterator a -> Option (a, Iterator a)`
    - Get next value and next iterator.
- `push_front : a -> Iterator a -> Iterator a`
    - Append an element to an iterator.
- `reverse : Iterator a -> Iterator a`
    - Reverse an iterator.
- `take : Int -> Iterator a -> Iterator a`
    - Take at most n elements from an iterator.
- `zip : Iterator a -> Iterator b -> Iterator (a, b)`
    - Zip two iterators.

Implementing Traits:

- `Iterator a : Add`
    - Adds two iterators by `Iterator.append`.
- `[a : Eq] Iterator a : Eq`

### Std.Option

`Option a` contains a value of type `a`, or contains nothing.

```
type Option a = union { none: (), some: a };
```

Methods:

- `map : (a -> b) -> Option a -> Option b`
    - Apply a function to the contained value. If the option is `none()`, do nothing.
- `unwrap : Option a -> a`
    - Exctract the contained value. If the option is `none()`, this function panics.

### Std.String

The type of strings.

Methods:

- `concat : String -> String -> String`
    - Concatenate two strings.
    - Note: Since `s1.concat(s2)` puts `s2` after `s1`, `concat(lhs, rhs)` puts `lhs` after `rhs`.
- `join : String -> Iterator String -> String`
    - Join strings by a separator.
    - Example: `Iterator.from_array(["a", "b", "c"]).join(", ") == "a, b, c"`
- `concat_iter : Iterator String -> String`
    - Concatenate an iterator of strings.
- `get_length : String -> Int`
    - Returns the length of the string.

Implementing Traits:

- `String : Add`
    - Add two strings by `String.concat`.
- `String : Eq`

## Functions

### Std.fix : ((a -> b) -> a -> b) -> a -> b

`fix` enables you to make a recursive function locally. The idiom is: `fix $ |loop, var| -> (expression calls loop)`.

```
module Main;

main : IOState -> ((), IOState);
main = (
    let fact = fix $ |loop, n| if n == 0 then 1 else n * loop (n-1);
    print! $ fact(5).to_string // evaluates to 5 * 4 * 3 * 2 * 1 = 120
);
```

### Std.loop : s -> (s -> LoopResult s r) -> r

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
    print! $ sum.to_string
); // evaluates to 0 + 1 + ... + 99 
```

### Std.Debug.debug_print : String -> ()

### Std.Debug.debug_println : String -> ()

### Std.Debug.abort : () -> a

### Std.Debug.assert : String -> Bool -> ()

### Std.Debug.assert_eq : [a: Eq] String -> a -> a -> ()

## Traits

### Std.ToString

- `to_string : [a: ToString] a -> String`

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