# Std

Module `Std` provides basic types, traits and values.

This module is special in the sense that:
- It is always imported implicitly. If you don't want to import some or all of entities in this module, you should write `import Std {...entities...}` explicitly.
- It contains built-in types or values which are defined or implemented directly by Fix compiler, not by Fix source code.

NOTE on tuples:
The tuple types `Std::TupleN` are defined on demand, i.e., if the user uses N-tuple in the source code,
the compiler generates definition `TupleN` and related functions / trait implementations.
The document for `Std` module describes about them up to N=3, but you can use larger tuples in the same way.

## Values

### namespace Std

#### compose

Type: `(a -> b) -> (b -> c) -> a -> c`

Composes two functions. Composition operators `<<` and `>>` is translated to use of `compose`.

#### fix

Type: `((a -> b) -> a -> b) -> a -> b`

`fix` enables you to make a recursive function locally.

The idiom is `fix $ |loop, arg| -> {loop_body}`. In `{loop_body}`, you can call `loop` to make a recursion.

Example:
```
module Main;

main : IO ();
main = (
    let fact = fix $ |loop, n| if n == 0 { 1 } else { n * loop (n-1) };
    println $ fact(5).to_string // evaluates to 5 * 4 * 3 * 2 * 1 = 120
);
```

#### loop

Type: `s -> (s -> Std::LoopState s r) -> r`

`loop` enables you to make a loop. `LoopState` is a union type defined as follows:

```
type LoopState s r = unbox union { continue : s, break : r };
```

`loop` takes two arguments: the initial state of the loop `s0` and the loop body function `body`.
It first calls `body` on `s0`.
If `body` returns `break(r)`, then the loop ends and returns `r` as the result.
If `body` returns `continue(s)`, then the loop calls again `body` on `s`.

Example:
```
module Main;

main : IO ();
main = (
    let sum = loop((0, 0), |(i, sum)|
        if i == 100 { break $ sum };
        continue $ (i + 1, sum + i)
    );
    println $ sum.to_string
); // evaluates to 0 + 1 + ... + 99
```

#### loop_m

Type: `[m : Std::Monad] s -> (s -> m (Std::LoopState s r)) -> m r`

Monadic loop function. This is similar to `loop` but can be used to perform monadic action at each loop.

It is convenient to use `continue_m` and `break_m` to create monadic loop body function.

The following program prints "Hello World! (i)" for i = 0, 1, 2.

```
module Main;

main : IO ();
main = (
    loop_m(0, |i| (
        if i == 3 { break_m $ () };
        println("Hello World! (" + i.to_string + ")");;
        continue_m $ i + 1
    ))
);
```

#### mark_threaded

Type: `a -> a`

Traverses all values reachable from the given value, and changes the reference counters of them into multi-threaded mode.

#### undefined

Type: `Std::String -> a`

Generates an undefined value.

Calling this function prints `msg` to the stderr, flush stderr, and aborts the program (calls `abort` in libc).
Since `undefined(msg)` has generic type `a`, you can put it anywhere and it will be type-checked.

This is useful when you want to write a placeholder that will be implemented later:

```
truth : I64;
truth = undefined("I will implement the truth later.");
```

Another use case is aborting the program when a certain branch of the code should not be reached:

```
if condition {
    // Do something.
} else {
    undefined("This branch should not be reached.");
}
```

#### unsafe_is_unique

Type: `a -> (Std::Bool, a)`

This function checks if a value is uniquely referenced by a name, and returns the result paired with the given value itself. An unboxed value is always considered unique.

NOTE: Changing outputs of your function depending on uniqueness breaks the referential transparency of the function. If you want to assert that a value is unique, consider using `Debug::assert_unique` instead.

Example: 

```
module Main;


main : IO ();
main = (
    // For unboxed value, it returns true even if the value is used later.
    let int_val = 42;
    let (unique, _) = int_val.unsafe_is_unique;
    let use = int_val + 1;
    assert_eq(|_|"fail: int_val is shared", unique, true);;

    // For boxed value, it returns true if the value isn't used later.
    let arr = Array::fill(10, 10);
    let (unique, arr) = arr.unsafe_is_unique;
    let use = arr.@(0); // This `arr` is not the one passed to `is_unique`, but the one returned by `is_unique`.
    assert_eq(|_|"fail: arr is shared", unique, true);;

    // Fox boxed value, it returns false if the value will be used later.
    let arr = Array::fill(10, 10);
    let (unique, _) = arr.unsafe_is_unique;
    let use = arr.@(0);
    assert_eq(|_|"fail: arr is unique", unique, false);;

    pure()
);
```

#### with_retained

Type: `(a -> b) -> a -> b`

`x.with_retained(f)` runs `f` with retained `x`. 
It is guaranteed that `x` is keep alive until `with_retained` is finished, even after `f` has finished using `x` in it. 

A typical use case of this function is the implementation of `Std::Array::borrow_ptr`.

### namespace Std::Add

#### add

Type: `[a : Std::Add] a -> a -> a`

Adds two values. An expression `x + y` is translated to `add(x, y)`.

### namespace Std::Array

#### @

Type: `Std::I64 -> Std::Array a -> a`

Gets an element of an array at the specified index.

#### _get_ptr

Type: `Std::Array a -> Std::Ptr`

Get the pointer to the memory region where elements are stored.

This function is dangerous because if the array is not used after call of this function, the array will be deallocated soon and the returned pointer will be dangling.
Try using `borrow_ptr` instead.

@deprecated
Use `Std::FFI::_get_boxed_ptr` instead.

#### _get_sub_size_with_length_and_additional_capacity

Type: `Std::I64 -> Std::I64 -> Std::I64 -> Std::I64 -> Std::Array a -> Std::Array a`

A function like `get_sub`, but behaves as if the size of the array is the specified value,
and has a parameter to specify additional capacity of the returned `Array`.

#### _sort_range_using_buffer

Type: `Std::Array a -> Std::I64 -> Std::I64 -> ((a, a) -> Std::Bool) -> Std::Array a -> (Std::Array a, Std::Array a)`

Sorts elements in a range of a vector by "less than" comparator.

This function receives a working buffer as the first argument to reduce memory allocation, and returns it as second element.

#### _unsafe_force_unique

Type: `Std::Array a -> Std::Array a`

Force the uniqueness of an array.
If the given array is shared, this function returns the cloned array.

DEPRECATED:
This function is unsafe and deprecated because it is fragile when the "common expression elimination" optimization is implemented in the future. 
Consider the following example:

```
f : Array a -> Array a
f = |arr| arr.force_unique.do_something_for_unique_array;

let x = [1, 2, 3];
let y = f(x);
let z = f(x);
```

When this function `f` is inlined, the code will be as follows.

```
let x = [1, 2, 3];
let y = x.force_unique.do_something_for_unique_array;
let z = x.force_unique.do_something_for_unique_array;
```

Here, if the optimization is applied to the two `x.force_unique`, the code will call `do_something_for_unique_array` with a non-unique array.

```
let x = [1, 2, 3];
let x = x.force_unique;
let y = x.do_something_for_unique_array; // Here `x` is not unique
let z = x.do_something_for_unique_array;
```

Therefore, to use this function safely, you need to suppress the inlining of the above `f`. It is uncertain whether a function attribute such as "noinline" will be added in the future, so this function is deprecated currently.

#### _unsafe_get

Type: `Std::I64 -> Std::Array a -> a`

Gets a value from an array and returns it paired with the array itself, without bounds checking and retaining the value.

#### _unsafe_get_linear

Type: `Std::I64 -> Std::Array a -> (Std::Array a, a)`

Gets a value from an array, without bounds checking and retaining the returned value.

#### _unsafe_set

Type: `Std::I64 -> a -> Std::Array a -> Std::Array a`

Sets a value into an array, without uniqueness checking, bounds checking and releasing the old value.

#### _unsafe_set_size

Type: `Std::I64 -> Std::Array a -> Std::Array a`

Updates the length of an array, without uniqueness checking or validation of the given length value.

#### act

Type: `[f : Std::Functor] Std::I64 -> (a -> f a) -> Std::Array a -> f (Std::Array a)`

Modifies an array by a functorial action.

Semantically, `arr.act(idx, fun)` is equivalent to `fun(arr.@(idx)).map(|elm| arr.set(idx, elm))`.

This function can be defined for any functor `f` in general, but it is easier to understand the behavior when `f` is a monad:
the monadic action `act(idx, fun, arr)` first performs `fun(arr.@(idx))` to get a value `elm`, and returns a pure value `arr.set(idx, elm)`.

If you call `arr.act(idx, fun)` when both of `arr` and `arr.@(idx)` are unique, it is assured that `fun` receives the unique value.

If you call `act` on an array which is shared, this function clones the given array when inserting the result of your action into the array.
This means that you don't need to pay cloning cost when your action failed, as expected.

#### append

Type: `Std::Array a -> Std::Array a -> Std::Array a`

Appends an array to an array.

Note: Since `a1.append(a2)` puts `a2` after `a1`, `append(lhs, rhs)` puts `lhs` after `rhs`.

#### empty

Type: `Std::I64 -> Std::Array a`

Creates an empty array with specified capacity.

#### fill

Type: `Std::I64 -> a -> Std::Array a`

Creates an array of the specified length filled with the initial value.

The capacity is set to the same value as the length.

Example: `fill(n, x) == [x, x, x, ..., x]` (of length `n`).

#### find_by

Type: `(a -> Std::Bool) -> Std::Array a -> Std::Option Std::I64`

Finds the first index at which the element satisfies a condition.

#### from_iter

Type: `[it : Std::Iterator, Std::Iterator::Item it = a] it -> Std::Array a`

Create an array from an iterator.

#### from_map

Type: `Std::I64 -> (Std::I64 -> a) -> Std::Array a`

Creates an array by a mapping function.

#### get_capacity

Type: `Std::Array a -> Std::I64`

Gets the capacity of an array.

#### get_first

Type: `Std::Array a -> Std::Option a`

Gets the first element of an array. Returns none if the array is empty.

#### get_last

Type: `Std::Array a -> Std::Option a`

Gets the last element of an array. Returns none if the array is empty.

#### get_size

Type: `Std::Array a -> Std::I64`

Gets the length of an array.

#### get_sub

Type: `Std::I64 -> Std::I64 -> Std::Array a -> Std::Array a`

`arr.get_sub(s, e)` returns an array `[ arr.@(i) | i ∈ [s, e) ]`.

`s` and `e` are clamped to the range `[0, arr.get_size]`.

#### is_empty

Type: `Std::Array a -> Std::Bool`

Returns if the array is empty

#### mod

Type: `Std::I64 -> (a -> a) -> Std::Array a -> Std::Array a`

Updates an array by applying a function to the element at the specified index.

This function clones the given array if it is shared.

If you call `arr.mod(i, f)` when both of `arr` and `arr.@(i)` are unique, it is assured that `f` receives the element value which is unique.

#### pop_back

Type: `Std::Array a -> Std::Array a`

Pops an element at the back of an array.
If the array is empty, this function does nothing.

#### push_back

Type: `a -> Std::Array a -> Std::Array a`

Pushes an element to the back of an array.

#### reserve

Type: `Std::I64 -> Std::Array a -> Std::Array a`

Reserves the memory region for an array.

#### set

Type: `Std::I64 -> a -> Std::Array a -> Std::Array a`

Updates an array by setting a value as the element at the specified index.

This function clones the given array if it is shared.

#### sort_by

Type: `((a, a) -> Std::Bool) -> Std::Array a -> Std::Array a`

Sorts elements in a vector by "less than" comparator.

#### to_iter

Type: `Std::Array a -> Std::Iterator::ArrayIterator a`

Converts an array to an iterator.

#### truncate

Type: `Std::I64 -> Std::Array a -> Std::Array a`

Truncates an array, keeping the given number of first elements.

`truncante(len, arr)` does nothing if `len >= arr.get_size`.

### namespace Std::Box

#### make

Type: `a -> Std::Box a`

### namespace Std::Debug

#### _debug_print_to_stream

Type: `Std::IO::IOHandle -> Std::String -> ()`

Prints a string to the specified stream and flushes the stream.

NOTE: This function is not pure and should only be used for temporary debugging purposes.

#### assert

Type: `Std::Lazy Std::String -> Std::Bool -> Std::IO ()`

Asserts that a condition (boolean value) is true.

If the assertion failed, prints a message to the stderr and aborts the program.

#### assert_eq

Type: `[a : Std::Eq] Std::Lazy Std::String -> a -> a -> Std::IO ()`

Asserts that two values are equal.

If the assertion failed, prints a message to the stderr and aborts the program.

#### assert_unique

Type: `Std::Lazy Std::String -> a -> a`

Asserts that the given value is unique, and returns the given value.
If the assertion failed, prints a message to the stderr and aborts the program.

The main use of this function is to check whether a boxed value given as an argument is unique.

#### consumed_time_while_io

Type: `Std::IO a -> Std::IO (a, Std::F64)`

Get clocks (cpu time) elapsed while executing an I/O action.

#### consumed_time_while_lazy

Type: `Std::Lazy a -> (a, Std::F64)`

Get clocks (cpu time) elapsed while evaluating a lazy value.

NOTE: This function is not pure and should only be used for temporary debugging purposes.

#### debug_eprint

Type: `Std::String -> ()`

Prints a string to stderr and flushes.

NOTE: This function is not pure and should only be used for temporary debugging purposes.

#### debug_eprintln

Type: `Std::String -> ()`

Prints a string followed by a newline to stderr and flushes.

NOTE: This function is not pure and should only be used for temporary debugging purposes.

#### debug_print

Type: `Std::String -> ()`

Prints a string to stdout and flushes.

NOTE: This function is not pure and should only be used for temporary debugging purposes.

#### debug_println

Type: `Std::String -> ()`

Prints a string followed by a newline to stdout and flushes.

NOTE: This function is not pure and should only be used for temporary debugging purposes.

### namespace Std::Div

#### div

Type: `[a : Std::Div] a -> a -> a`

Divides a value by another value. An expression `x / y` is translated to `div(x, y)`.

### namespace Std::Eq

#### eq

Type: `[a : Std::Eq] a -> a -> Std::Bool`

Checks equality of two values. An expression `x == y` is translated to `eq(x, y)`.

### namespace Std::F32

#### abs

Type: `Std::F32 -> Std::F32`

#### infinity

Type: `Std::F32`

The infinity value for the given floating point type.

#### quiet_nan

Type: `Std::F32`

A floating number represented by `01...1` in binary.

#### to_CChar

Type: `Std::F32 -> Std::I8`

Casts a value of `F32` into a value of `CChar`.

#### to_CDouble

Type: `Std::F32 -> Std::F64`

Casts a value of `F32` into a value of `CDouble`.

#### to_CFloat

Type: `Std::F32 -> Std::F32`

Casts a value of `F32` into a value of `CFloat`.

#### to_CInt

Type: `Std::F32 -> Std::I32`

Casts a value of `F32` into a value of `CInt`.

#### to_CLong

Type: `Std::F32 -> Std::I64`

Casts a value of `F32` into a value of `CLong`.

#### to_CLongLong

Type: `Std::F32 -> Std::I64`

Casts a value of `F32` into a value of `CLongLong`.

#### to_CShort

Type: `Std::F32 -> Std::I16`

Casts a value of `F32` into a value of `CShort`.

#### to_CSizeT

Type: `Std::F32 -> Std::U64`

Casts a value of `F32` into a value of `CSizeT`.

#### to_CUnsignedChar

Type: `Std::F32 -> Std::U8`

Casts a value of `F32` into a value of `CUnsignedChar`.

#### to_CUnsignedInt

Type: `Std::F32 -> Std::U32`

Casts a value of `F32` into a value of `CUnsignedInt`.

#### to_CUnsignedLong

Type: `Std::F32 -> Std::U64`

Casts a value of `F32` into a value of `CUnsignedLong`.

#### to_CUnsignedLongLong

Type: `Std::F32 -> Std::U64`

Casts a value of `F32` into a value of `CUnsignedLongLong`.

#### to_CUnsignedShort

Type: `Std::F32 -> Std::U16`

Casts a value of `F32` into a value of `CUnsignedShort`.

#### to_F32

Type: `Std::F32 -> Std::F32`

Casts a value of `F32` into a value of `F32`.

#### to_F64

Type: `Std::F32 -> Std::F64`

Casts a value of `F32` into a value of `F64`.

#### to_I16

Type: `Std::F32 -> Std::I16`

Casts a value of `F32` into a value of `I16`.

#### to_I32

Type: `Std::F32 -> Std::I32`

Casts a value of `F32` into a value of `I32`.

#### to_I64

Type: `Std::F32 -> Std::I64`

Casts a value of `F32` into a value of `I64`.

#### to_I8

Type: `Std::F32 -> Std::I8`

Casts a value of `F32` into a value of `I8`.

#### to_U16

Type: `Std::F32 -> Std::U16`

Casts a value of `F32` into a value of `U16`.

#### to_U32

Type: `Std::F32 -> Std::U32`

Casts a value of `F32` into a value of `U32`.

#### to_U64

Type: `Std::F32 -> Std::U64`

Casts a value of `F32` into a value of `U64`.

#### to_U8

Type: `Std::F32 -> Std::U8`

Casts a value of `F32` into a value of `U8`.

#### to_string_exp

Type: `Std::F32 -> Std::String`

Converts a floating number to a string of exponential form.

#### to_string_exp_precision

Type: `Std::U8 -> Std::F32 -> Std::String`

Converts a floating number to a string of exponential form with specified precision (i.e., number of digits after the decimal point).

#### to_string_precision

Type: `Std::U8 -> Std::F32 -> Std::String`

Converts a floating number to a string with specified precision (i.e., number of digits after the decimal point).

### namespace Std::F64

#### abs

Type: `Std::F64 -> Std::F64`

#### infinity

Type: `Std::F64`

The infinity value for the given floating point type.

#### quiet_nan

Type: `Std::F64`

A floating number represented by `01...1` in binary.

#### to_CChar

Type: `Std::F64 -> Std::I8`

Casts a value of `F64` into a value of `CChar`.

#### to_CDouble

Type: `Std::F64 -> Std::F64`

Casts a value of `F64` into a value of `CDouble`.

#### to_CFloat

Type: `Std::F64 -> Std::F32`

Casts a value of `F64` into a value of `CFloat`.

#### to_CInt

Type: `Std::F64 -> Std::I32`

Casts a value of `F64` into a value of `CInt`.

#### to_CLong

Type: `Std::F64 -> Std::I64`

Casts a value of `F64` into a value of `CLong`.

#### to_CLongLong

Type: `Std::F64 -> Std::I64`

Casts a value of `F64` into a value of `CLongLong`.

#### to_CShort

Type: `Std::F64 -> Std::I16`

Casts a value of `F64` into a value of `CShort`.

#### to_CSizeT

Type: `Std::F64 -> Std::U64`

Casts a value of `F64` into a value of `CSizeT`.

#### to_CUnsignedChar

Type: `Std::F64 -> Std::U8`

Casts a value of `F64` into a value of `CUnsignedChar`.

#### to_CUnsignedInt

Type: `Std::F64 -> Std::U32`

Casts a value of `F64` into a value of `CUnsignedInt`.

#### to_CUnsignedLong

Type: `Std::F64 -> Std::U64`

Casts a value of `F64` into a value of `CUnsignedLong`.

#### to_CUnsignedLongLong

Type: `Std::F64 -> Std::U64`

Casts a value of `F64` into a value of `CUnsignedLongLong`.

#### to_CUnsignedShort

Type: `Std::F64 -> Std::U16`

Casts a value of `F64` into a value of `CUnsignedShort`.

#### to_F32

Type: `Std::F64 -> Std::F32`

Casts a value of `F64` into a value of `F32`.

#### to_F64

Type: `Std::F64 -> Std::F64`

Casts a value of `F64` into a value of `F64`.

#### to_I16

Type: `Std::F64 -> Std::I16`

Casts a value of `F64` into a value of `I16`.

#### to_I32

Type: `Std::F64 -> Std::I32`

Casts a value of `F64` into a value of `I32`.

#### to_I64

Type: `Std::F64 -> Std::I64`

Casts a value of `F64` into a value of `I64`.

#### to_I8

Type: `Std::F64 -> Std::I8`

Casts a value of `F64` into a value of `I8`.

#### to_U16

Type: `Std::F64 -> Std::U16`

Casts a value of `F64` into a value of `U16`.

#### to_U32

Type: `Std::F64 -> Std::U32`

Casts a value of `F64` into a value of `U32`.

#### to_U64

Type: `Std::F64 -> Std::U64`

Casts a value of `F64` into a value of `U64`.

#### to_U8

Type: `Std::F64 -> Std::U8`

Casts a value of `F64` into a value of `U8`.

#### to_string_exp

Type: `Std::F64 -> Std::String`

Converts a floating number to a string of exponential form.

#### to_string_exp_precision

Type: `Std::U8 -> Std::F64 -> Std::String`

Converts a floating number to a string of exponential form with specified precision (i.e., number of digits after the decimal point).

#### to_string_precision

Type: `Std::U8 -> Std::F64 -> Std::String`

Converts a floating number to a string with specified precision (i.e., number of digits after the decimal point).

### namespace Std::FFI

#### _get_boxed_ptr

Type: `[a : Std::Boxed] a -> Std::Ptr`

Returns a pointer to the data of a boxed value.

The returned pointer points to:
- if the value is an `Array`, the first element of the array,
- if the value is a struct, the first field,
- if the value is an union, the data field (not the tag field).

The difference from `boxed_to_retained_ptr` is that this function returns a pointer to region where the payload of a boxed value is stored;
on the other hand, `boxed_to_retained_ptr` returns a pointer to the boxed value itself (which currently points to the reference counter of the boxed value).

NOTE: 
This function is unsafe in that if the call `v._get_boxed_ptr` is the last usage of `v`, then this function deallocates `v` and returns a dangling pointer.
To avoid this issue, use `borrow_boxed`, `borrow_boxed_io`, `mutate_boxed`, or `mutate_boxed_io` instead.

#### borrow_boxed

Type: `[a : Std::Boxed] (Std::Ptr -> b) -> a -> b`

Borrows a pointer to the data of a boxed value.

For the details of the pointer, see the document of `_get_boxed_ptr`.

#### borrow_boxed_io

Type: `[a : Std::Boxed] (Std::Ptr -> Std::IO b) -> a -> Std::IO b`

Performs an IO action borrowing a pointer to the data of a boxed value.

For the details of the pointer, see the document of `_get_boxed_ptr`.

#### boxed_from_retained_ptr

Type: `[a : Std::Boxed] Std::Ptr -> a`

Creates a boxed value from a retained pointer obtained by `boxed_to_retained_ptr`.

NOTE: 
It is the user's responsibility to ensure that the argument is actually a pointer to the type of the return value, and undefined behavior will occur if it is not.

#### boxed_to_retained_ptr

Type: `[a : Std::Boxed] a -> Std::Ptr`

Returns a retained pointer to a boxed value.
This function is used to share ownership of Fix's boxed values with foreign languages.

To get back the boxed value from the retained pointer, use `from_retained_ptr`.
To release / retain the value in a foreign language, call the function pointer obtained by `get_funptr_release` or `get_funptr_retain` on the pointer.

Note that the returned pointer points to the control block allocated by Fix, and does not necessary points to the data of the boxed value.
If you want to get a pointer to the data of the boxed value, use `borrow_boxed`.

#### clear_errno

Type: `Std::IO ()`

Sets errno to zero.

#### get_errno

Type: `Std::IO Std::FFI::CInt`

Gets errno which is set by C functions.

#### get_funptr_release

Type: `[a : Std::Boxed] Std::Lazy a -> Std::Ptr`

Returns a pointer to the function of type `void (*)(void*)` which releases a boxed value of type `a`.
This function is used to release a pointer obtained by `boxed_to_retained_ptr`.

Note that this function is requires a value of type `Lazy a`, not of `a`.
So you can get release function for a boxed type `T` even when you don't have a value of type `T` -- you can just use `|_| undefined("") : T`:

```
module Main;

type VoidType = box struct {};
// No constructor for `VoidType` is provided.

main: IO ();
main = (
    let release = (|_| undefined("") : VoidType).get_funptr_release; // Release function of `VoidType`.
    pure()
);
```

#### get_funptr_retain

Type: `[a : Std::Boxed] Std::Lazy a -> Std::Ptr`

Returns a pointer to the function of type `void (*)(void*)` which retains a boxed value of type `a`.
This function is used to retain a pointer obtained by `boxed_to_retained_ptr`.

For the reason that this function requires a value of type `Lazy a`, not of `a`, see the document for `get_funptr_release`.

#### mutate_boxed

Type: `[a : Std::Boxed] (Std::Ptr -> Std::IO b) -> a -> (a, b)`

`x.mutate_boxed(io)` gets a pointer `ptr` to the data that `x` points to, executes `io(ptr)`, and then returns mutated `x` paired with the result of ``io(ptr)``.

The IO action `io(ptr)` is expected to modify the value of `x` through the obtained pointer. 
Do not perform any IO operations other than mutating the value of `x`.

For more details on the value of the pointer passed to `io`, see the document of `_get_boxed_ptr`.

This function first clones the value if `x` is not unique.

#### mutate_boxed_io

Type: `[a : Std::Boxed] (Std::Ptr -> Std::IO b) -> a -> Std::IO (a, b)`

`x.mutate_boxed_io(io)` gets a pointer `ptr` to the data that `x` points to, executes `io(ptr)`, and then returns mutated `x` paired with the result of `io(ptr)`.

Similar to `mutate_boxed`, but this function is used when you want to run the IO action in the existing IO context.

For more details, see the document of `mutate_boxed`.

#### mutate_boxed_ios

Type: `[a : Std::Boxed] (Std::Ptr -> Std::IO b) -> a -> Std::IO::IOState -> (Std::IO::IOState, (a, b))`

Internal implementation of the `mutate_boxed_io` function.

### namespace Std::FFI::Destructor

#### borrow

Type: `(a -> b) -> Std::FFI::Destructor a -> b`

Borrow the contained value.

`borrow(worker, dtor)` calls `worker` on the contained value captured by `dtor`, and returns the value returned by `worker`.

It is guaranteed that the `dtor` is alive during the call of `worker`.
In other words, the `worker` receives the contained value for which the destructor is not called yet.

#### borrow_io

Type: `(a -> Std::IO b) -> Std::FFI::Destructor a -> Std::IO b`

Performs an IO action borrowing the contained value.

#### make

Type: `a -> (a -> Std::IO a) -> Std::FFI::Destructor a`

Make a destructor value.

#### mutate_unique

Type: `(a -> Std::IO a) -> (a -> Std::IO b) -> Std::FFI::Destructor a -> (Std::FFI::Destructor a, b)`

Apply an IO action which mutates the semantics of the value.

`dtor.mutate_unique(ctor, action)` applies `action` to `dtor` if `dtor` is unique.
If `dtor` is shared, it creates a new `Destructor` value using `ctor` and applies `action` to the new value.

The `action` is allowed to modify the external resource stored in `dtor` (e.g., if `value` is a pointer, it can modify the value pointed by the pointer).
Also, `ctor` should be a "copy constructor" (e.g., memcpy) of the external resource stored in `dtor`.

#### mutate_unique_io

Type: `(a -> Std::IO a) -> (a -> Std::IO b) -> Std::FFI::Destructor a -> Std::IO (Std::FFI::Destructor a, b)`

Apply an IO action which mutates the semantics of the value.

This is similar to `mutate_unique`, but the `ctor` and `action` is executed in the context of the external `IO` context.

### namespace Std::FromBytes

#### from_bytes

Type: `[a : Std::FromBytes] Std::Array Std::U8 -> Std::Result Std::ErrMsg a`

### namespace Std::FromString

#### from_string

Type: `[a : Std::FromString] Std::String -> Std::Result Std::ErrMsg a`

### namespace Std::Functor

#### forget

Type: `[f : Std::Functor] f a -> f ()`

#### map

Type: `[f : Std::Functor] (a -> b) -> f a -> f b`

### namespace Std::I16

#### abs

Type: `Std::I16 -> Std::I16`

#### bit_and

Type: `Std::I16 -> Std::I16 -> Std::I16`

Calculates bitwise AND of two values.

#### bit_or

Type: `Std::I16 -> Std::I16 -> Std::I16`

Calculates bitwise OR of two values.

#### bit_xor

Type: `Std::I16 -> Std::I16 -> Std::I16`

Calculates bitwise XOR of two values.

#### maximum

Type: `Std::I16`

#### minimum

Type: `Std::I16`

#### shift_left

Type: `Std::I16 -> Std::I16 -> Std::I16`

`v.shift_left(w)` shifts `v` to left by `w` bits.

#### shift_right

Type: `Std::I16 -> Std::I16 -> Std::I16`

`v.shift_right(w)` shifts `v` to right by `w` bits.

#### to_CChar

Type: `Std::I16 -> Std::I8`

Casts a value of `I16` into a value of `CChar`.

#### to_CDouble

Type: `Std::I16 -> Std::F64`

Casts a value of `I16` into a value of `CDouble`.

#### to_CFloat

Type: `Std::I16 -> Std::F32`

Casts a value of `I16` into a value of `CFloat`.

#### to_CInt

Type: `Std::I16 -> Std::I32`

Casts a value of `I16` into a value of `CInt`.

#### to_CLong

Type: `Std::I16 -> Std::I64`

Casts a value of `I16` into a value of `CLong`.

#### to_CLongLong

Type: `Std::I16 -> Std::I64`

Casts a value of `I16` into a value of `CLongLong`.

#### to_CShort

Type: `Std::I16 -> Std::I16`

Casts a value of `I16` into a value of `CShort`.

#### to_CSizeT

Type: `Std::I16 -> Std::U64`

Casts a value of `I16` into a value of `CSizeT`.

#### to_CUnsignedChar

Type: `Std::I16 -> Std::U8`

Casts a value of `I16` into a value of `CUnsignedChar`.

#### to_CUnsignedInt

Type: `Std::I16 -> Std::U32`

Casts a value of `I16` into a value of `CUnsignedInt`.

#### to_CUnsignedLong

Type: `Std::I16 -> Std::U64`

Casts a value of `I16` into a value of `CUnsignedLong`.

#### to_CUnsignedLongLong

Type: `Std::I16 -> Std::U64`

Casts a value of `I16` into a value of `CUnsignedLongLong`.

#### to_CUnsignedShort

Type: `Std::I16 -> Std::U16`

Casts a value of `I16` into a value of `CUnsignedShort`.

#### to_F32

Type: `Std::I16 -> Std::F32`

Casts a value of `I16` into a value of `F32`.

#### to_F64

Type: `Std::I16 -> Std::F64`

Casts a value of `I16` into a value of `F64`.

#### to_I16

Type: `Std::I16 -> Std::I16`

Casts a value of `I16` into a value of `I16`.

#### to_I32

Type: `Std::I16 -> Std::I32`

Casts a value of `I16` into a value of `I32`.

#### to_I64

Type: `Std::I16 -> Std::I64`

Casts a value of `I16` into a value of `I64`.

#### to_I8

Type: `Std::I16 -> Std::I8`

Casts a value of `I16` into a value of `I8`.

#### to_U16

Type: `Std::I16 -> Std::U16`

Casts a value of `I16` into a value of `U16`.

#### to_U32

Type: `Std::I16 -> Std::U32`

Casts a value of `I16` into a value of `U32`.

#### to_U64

Type: `Std::I16 -> Std::U64`

Casts a value of `I16` into a value of `U64`.

#### to_U8

Type: `Std::I16 -> Std::U8`

Casts a value of `I16` into a value of `U8`.

### namespace Std::I32

#### abs

Type: `Std::I32 -> Std::I32`

#### bit_and

Type: `Std::I32 -> Std::I32 -> Std::I32`

Calculates bitwise AND of two values.

#### bit_or

Type: `Std::I32 -> Std::I32 -> Std::I32`

Calculates bitwise OR of two values.

#### bit_xor

Type: `Std::I32 -> Std::I32 -> Std::I32`

Calculates bitwise XOR of two values.

#### maximum

Type: `Std::I32`

#### minimum

Type: `Std::I32`

#### shift_left

Type: `Std::I32 -> Std::I32 -> Std::I32`

`v.shift_left(w)` shifts `v` to left by `w` bits.

#### shift_right

Type: `Std::I32 -> Std::I32 -> Std::I32`

`v.shift_right(w)` shifts `v` to right by `w` bits.

#### to_CChar

Type: `Std::I32 -> Std::I8`

Casts a value of `I32` into a value of `CChar`.

#### to_CDouble

Type: `Std::I32 -> Std::F64`

Casts a value of `I32` into a value of `CDouble`.

#### to_CFloat

Type: `Std::I32 -> Std::F32`

Casts a value of `I32` into a value of `CFloat`.

#### to_CInt

Type: `Std::I32 -> Std::I32`

Casts a value of `I32` into a value of `CInt`.

#### to_CLong

Type: `Std::I32 -> Std::I64`

Casts a value of `I32` into a value of `CLong`.

#### to_CLongLong

Type: `Std::I32 -> Std::I64`

Casts a value of `I32` into a value of `CLongLong`.

#### to_CShort

Type: `Std::I32 -> Std::I16`

Casts a value of `I32` into a value of `CShort`.

#### to_CSizeT

Type: `Std::I32 -> Std::U64`

Casts a value of `I32` into a value of `CSizeT`.

#### to_CUnsignedChar

Type: `Std::I32 -> Std::U8`

Casts a value of `I32` into a value of `CUnsignedChar`.

#### to_CUnsignedInt

Type: `Std::I32 -> Std::U32`

Casts a value of `I32` into a value of `CUnsignedInt`.

#### to_CUnsignedLong

Type: `Std::I32 -> Std::U64`

Casts a value of `I32` into a value of `CUnsignedLong`.

#### to_CUnsignedLongLong

Type: `Std::I32 -> Std::U64`

Casts a value of `I32` into a value of `CUnsignedLongLong`.

#### to_CUnsignedShort

Type: `Std::I32 -> Std::U16`

Casts a value of `I32` into a value of `CUnsignedShort`.

#### to_F32

Type: `Std::I32 -> Std::F32`

Casts a value of `I32` into a value of `F32`.

#### to_F64

Type: `Std::I32 -> Std::F64`

Casts a value of `I32` into a value of `F64`.

#### to_I16

Type: `Std::I32 -> Std::I16`

Casts a value of `I32` into a value of `I16`.

#### to_I32

Type: `Std::I32 -> Std::I32`

Casts a value of `I32` into a value of `I32`.

#### to_I64

Type: `Std::I32 -> Std::I64`

Casts a value of `I32` into a value of `I64`.

#### to_I8

Type: `Std::I32 -> Std::I8`

Casts a value of `I32` into a value of `I8`.

#### to_U16

Type: `Std::I32 -> Std::U16`

Casts a value of `I32` into a value of `U16`.

#### to_U32

Type: `Std::I32 -> Std::U32`

Casts a value of `I32` into a value of `U32`.

#### to_U64

Type: `Std::I32 -> Std::U64`

Casts a value of `I32` into a value of `U64`.

#### to_U8

Type: `Std::I32 -> Std::U8`

Casts a value of `I32` into a value of `U8`.

### namespace Std::I64

#### abs

Type: `Std::I64 -> Std::I64`

#### bit_and

Type: `Std::I64 -> Std::I64 -> Std::I64`

Calculates bitwise AND of two values.

#### bit_or

Type: `Std::I64 -> Std::I64 -> Std::I64`

Calculates bitwise OR of two values.

#### bit_xor

Type: `Std::I64 -> Std::I64 -> Std::I64`

Calculates bitwise XOR of two values.

#### maximum

Type: `Std::I64`

#### minimum

Type: `Std::I64`

#### shift_left

Type: `Std::I64 -> Std::I64 -> Std::I64`

`v.shift_left(w)` shifts `v` to left by `w` bits.

#### shift_right

Type: `Std::I64 -> Std::I64 -> Std::I64`

`v.shift_right(w)` shifts `v` to right by `w` bits.

#### to_CChar

Type: `Std::I64 -> Std::I8`

Casts a value of `I64` into a value of `CChar`.

#### to_CDouble

Type: `Std::I64 -> Std::F64`

Casts a value of `I64` into a value of `CDouble`.

#### to_CFloat

Type: `Std::I64 -> Std::F32`

Casts a value of `I64` into a value of `CFloat`.

#### to_CInt

Type: `Std::I64 -> Std::I32`

Casts a value of `I64` into a value of `CInt`.

#### to_CLong

Type: `Std::I64 -> Std::I64`

Casts a value of `I64` into a value of `CLong`.

#### to_CLongLong

Type: `Std::I64 -> Std::I64`

Casts a value of `I64` into a value of `CLongLong`.

#### to_CShort

Type: `Std::I64 -> Std::I16`

Casts a value of `I64` into a value of `CShort`.

#### to_CSizeT

Type: `Std::I64 -> Std::U64`

Casts a value of `I64` into a value of `CSizeT`.

#### to_CUnsignedChar

Type: `Std::I64 -> Std::U8`

Casts a value of `I64` into a value of `CUnsignedChar`.

#### to_CUnsignedInt

Type: `Std::I64 -> Std::U32`

Casts a value of `I64` into a value of `CUnsignedInt`.

#### to_CUnsignedLong

Type: `Std::I64 -> Std::U64`

Casts a value of `I64` into a value of `CUnsignedLong`.

#### to_CUnsignedLongLong

Type: `Std::I64 -> Std::U64`

Casts a value of `I64` into a value of `CUnsignedLongLong`.

#### to_CUnsignedShort

Type: `Std::I64 -> Std::U16`

Casts a value of `I64` into a value of `CUnsignedShort`.

#### to_F32

Type: `Std::I64 -> Std::F32`

Casts a value of `I64` into a value of `F32`.

#### to_F64

Type: `Std::I64 -> Std::F64`

Casts a value of `I64` into a value of `F64`.

#### to_I16

Type: `Std::I64 -> Std::I16`

Casts a value of `I64` into a value of `I16`.

#### to_I32

Type: `Std::I64 -> Std::I32`

Casts a value of `I64` into a value of `I32`.

#### to_I64

Type: `Std::I64 -> Std::I64`

Casts a value of `I64` into a value of `I64`.

#### to_I8

Type: `Std::I64 -> Std::I8`

Casts a value of `I64` into a value of `I8`.

#### to_U16

Type: `Std::I64 -> Std::U16`

Casts a value of `I64` into a value of `U16`.

#### to_U32

Type: `Std::I64 -> Std::U32`

Casts a value of `I64` into a value of `U32`.

#### to_U64

Type: `Std::I64 -> Std::U64`

Casts a value of `I64` into a value of `U64`.

#### to_U8

Type: `Std::I64 -> Std::U8`

Casts a value of `I64` into a value of `U8`.

### namespace Std::I8

#### abs

Type: `Std::I8 -> Std::I8`

#### bit_and

Type: `Std::I8 -> Std::I8 -> Std::I8`

Calculates bitwise AND of two values.

#### bit_or

Type: `Std::I8 -> Std::I8 -> Std::I8`

Calculates bitwise OR of two values.

#### bit_xor

Type: `Std::I8 -> Std::I8 -> Std::I8`

Calculates bitwise XOR of two values.

#### maximum

Type: `Std::I8`

#### minimum

Type: `Std::I8`

#### shift_left

Type: `Std::I8 -> Std::I8 -> Std::I8`

`v.shift_left(w)` shifts `v` to left by `w` bits.

#### shift_right

Type: `Std::I8 -> Std::I8 -> Std::I8`

`v.shift_right(w)` shifts `v` to right by `w` bits.

#### to_CChar

Type: `Std::I8 -> Std::I8`

Casts a value of `I8` into a value of `CChar`.

#### to_CDouble

Type: `Std::I8 -> Std::F64`

Casts a value of `I8` into a value of `CDouble`.

#### to_CFloat

Type: `Std::I8 -> Std::F32`

Casts a value of `I8` into a value of `CFloat`.

#### to_CInt

Type: `Std::I8 -> Std::I32`

Casts a value of `I8` into a value of `CInt`.

#### to_CLong

Type: `Std::I8 -> Std::I64`

Casts a value of `I8` into a value of `CLong`.

#### to_CLongLong

Type: `Std::I8 -> Std::I64`

Casts a value of `I8` into a value of `CLongLong`.

#### to_CShort

Type: `Std::I8 -> Std::I16`

Casts a value of `I8` into a value of `CShort`.

#### to_CSizeT

Type: `Std::I8 -> Std::U64`

Casts a value of `I8` into a value of `CSizeT`.

#### to_CUnsignedChar

Type: `Std::I8 -> Std::U8`

Casts a value of `I8` into a value of `CUnsignedChar`.

#### to_CUnsignedInt

Type: `Std::I8 -> Std::U32`

Casts a value of `I8` into a value of `CUnsignedInt`.

#### to_CUnsignedLong

Type: `Std::I8 -> Std::U64`

Casts a value of `I8` into a value of `CUnsignedLong`.

#### to_CUnsignedLongLong

Type: `Std::I8 -> Std::U64`

Casts a value of `I8` into a value of `CUnsignedLongLong`.

#### to_CUnsignedShort

Type: `Std::I8 -> Std::U16`

Casts a value of `I8` into a value of `CUnsignedShort`.

#### to_F32

Type: `Std::I8 -> Std::F32`

Casts a value of `I8` into a value of `F32`.

#### to_F64

Type: `Std::I8 -> Std::F64`

Casts a value of `I8` into a value of `F64`.

#### to_I16

Type: `Std::I8 -> Std::I16`

Casts a value of `I8` into a value of `I16`.

#### to_I32

Type: `Std::I8 -> Std::I32`

Casts a value of `I8` into a value of `I32`.

#### to_I64

Type: `Std::I8 -> Std::I64`

Casts a value of `I8` into a value of `I64`.

#### to_I8

Type: `Std::I8 -> Std::I8`

Casts a value of `I8` into a value of `I8`.

#### to_U16

Type: `Std::I8 -> Std::U16`

Casts a value of `I8` into a value of `U16`.

#### to_U32

Type: `Std::I8 -> Std::U32`

Casts a value of `I8` into a value of `U32`.

#### to_U64

Type: `Std::I8 -> Std::U64`

Casts a value of `I8` into a value of `U64`.

#### to_U8

Type: `Std::I8 -> Std::U8`

Casts a value of `I8` into a value of `U8`.

### namespace Std::IO

#### _read_line_inner

Type: `Std::Bool -> Std::IO::IOHandle -> Std::IO::IOFail Std::String`

Reads characters from an IOHandle.

If the first argument `upto_newline` is true, this function reads a file upto newline or EOF.

#### close_file

Type: `Std::IO::IOHandle -> Std::IO ()`

Closes a file.

Unlike C's `fclose`, closing an already closed `IOHandle` is safe and does nothing.

#### eprint

Type: `Std::String -> Std::IO ()`

Prints a string to stderr.

#### eprintln

Type: `Std::String -> Std::IO ()`

Prints a string followed by a newline to stderr.

#### exit

Type: `Std::I64 -> Std::IO a`

Exits the program with an error code.

#### exit_with_msg

Type: `Std::I64 -> Std::String -> Std::IO a`

Exits the program with an error message and an error code.

The error message is written to the standard error output.

#### from_runner

Type: `(Std::IO::IOState -> (Std::IO::IOState, a)) -> Std::IO a`

Creates an IO action from a IO runner function, which is a function of type `IOState -> (IOState, a)`.

#### get_arg

Type: `Std::I64 -> Std::IO (Std::Option Std::String)`

`get_arg(n)` returns the n-th (0-indexed) command line argument.

If n is greater than or equal to the number of command line arguments, this function returns none.

#### get_arg_count

Type: `Std::IO Std::I64`

Gets the number of command line arguments.

#### get_args

Type: `Std::IO (Std::Array Std::String)`

Gets command line arguments.

#### input_line

Type: `Std::IO Std::String`

Reads a line from stdin. If some error occurr, this function aborts the program.

If you want to handle errors, use `read_line(stdin)` instead.

#### is_eof

Type: `Std::IO::IOHandle -> Std::IO Std::Bool`

Checks if an `IOHandle` reached to the EOF.

#### loop_lines

Type: `Std::IO::IOHandle -> s -> (s -> Std::String -> Std::LoopState s s) -> Std::IO::IOFail s`

Loop on lines read from an `IOHandle`.

`loop_lines(handle, initial_state, worker)` calls `worker` on the pair of current state and a line string read from `handle`.
The function `worker` should return an updated state as `LoopState` value, i.e., a value created by `continue` or `break`.
When the `handle` reaches to the EOF or `worker` returns a `break` value, `loop_lines` returns the last state value.

Note that the line string passed to `worker` may contain a newline code at the end. To remove it, use `String::strip_last_spaces`.

#### loop_lines_io

Type: `Std::IO::IOHandle -> s -> (s -> Std::String -> Std::IO::IOFail (Std::LoopState s s)) -> Std::IO::IOFail s`

Loop on lines read from an `IOHandle`.

Similar to `loop_lines`, but the worker function can perform an IO action.

#### open_file

Type: `Std::Path -> Std::String -> Std::IO::IOFail Std::IO::IOHandle`

Openes a file. The second argument is a mode string for `fopen` C function.

#### print

Type: `Std::String -> Std::IO ()`

Prints a string to stdout.

#### println

Type: `Std::String -> Std::IO ()`

Prints a string followed by a newline to stdout.

#### read_bytes

Type: `Std::IO::IOHandle -> Std::IO::IOFail (Std::Array Std::U8)`

Reads all bytes from an IOHandle.

#### read_file_bytes

Type: `Std::Path -> Std::IO::IOFail (Std::Array Std::U8)`

Reads all bytes from a file.

#### read_file_string

Type: `Std::Path -> Std::IO::IOFail Std::String`

Raads all characters from a file.

#### read_line

Type: `Std::IO::IOHandle -> Std::IO::IOFail Std::String`

Reads characters from a IOHandle upto newline or EOF.
The returned string may include newline at its end.

#### read_n_bytes

Type: `Std::IO::IOHandle -> Std::I64 -> Std::IO::IOFail (Std::Array Std::U8)`

Reads at most n bytes from an IOHandle.

#### read_string

Type: `Std::IO::IOHandle -> Std::IO::IOFail Std::String`

Reads all characters from an IOHandle.

#### stderr

Type: `Std::IO::IOHandle`

The handle for standard error.

#### stdin

Type: `Std::IO::IOHandle`

The handle for standard input.

#### stdout

Type: `Std::IO::IOHandle`

The handle for standard output.

#### unsafe_perform

Type: `Std::IO a -> a`

#### with_file

Type: `Std::Path -> Std::String -> (Std::IO::IOHandle -> Std::IO::IOFail a) -> Std::IO::IOFail a`

Performs a function with a file handle. The second argument is a mode string for `fopen` C function.

The file handle will be closed automatically.

#### write_bytes

Type: `Std::IO::IOHandle -> Std::Array Std::U8 -> Std::IO::IOFail ()`

Writes a byte array into an IOHandle.

#### write_file_bytes

Type: `Std::Path -> Std::Array Std::U8 -> Std::IO::IOFail ()`

Writes a byte array into a file.

#### write_file_string

Type: `Std::Path -> Std::String -> Std::IO::IOFail ()`

Writes a string into a file.

#### write_string

Type: `Std::IO::IOHandle -> Std::String -> Std::IO::IOFail ()`

Writes a string into an IOHandle.

### namespace Std::IO::IOFail

#### from_io_result

Type: `Std::IO (Std::Result Std::ErrMsg a) -> Std::IO::IOFail a`

Create from IO action of which returns `Result ErrMsg a`.

#### from_result

Type: `Std::Result Std::ErrMsg a -> Std::IO::IOFail a`

Creates an pure `IOFail` value from a `Result` value.

#### lift

Type: `Std::IO a -> Std::IO::IOFail a`

Lifts an `IO` action to a successful `IOFail` action.

#### throw

Type: `Std::ErrMsg -> Std::IO::IOFail a`

Creates an error `IOFail` action.

#### to_result

Type: `Std::IO::IOFail a -> Std::IO (Std::Result Std::ErrMsg a)`

Converts an `IOFail` to an `Result` value (wrapped by `IO`).

#### try

Type: `(Std::ErrMsg -> Std::IO a) -> Std::IO::IOFail a -> Std::IO a`

Converts an `IOFail` value to an `IO` value by an error handler (i.e., a `catch`) function.

### namespace Std::IO::IOHandle

#### _file_ptr

Type: `Std::IO::IOHandle -> Std::Ptr`

Gets pointer to C's `FILE` value from an `IOHandle`.

If the `IOHandle` is already closed, the function returns `nullptr`.

NOTE:
Do not directly close the file pointer by `fclose` or other functions.
Instead you should close `IOHandle` by `IO::close_file`.

DEPRECATED:
Use `get_file_ptr` instead.
This function is deprecated because it has a pure function interface, but the value of `_file_ptr` changes by calling `IO::close_file`.

#### from_file_ptr

Type: `Std::Ptr -> Std::IO::IOHandle`

Creates an `IOHandle` from a file pointer (i.e., pointer to C's `FILE`).

Creating two `IOHandle`s from a single file pointer is forbidden.

#### get_file_ptr

Type: `Std::IO::IOHandle -> Std::IO Std::Ptr`

Gets pointer to C's `FILE` value from an `IOHandle`.

If the `IOHandle` is already closed, the function returns `nullptr`.

NOTE:
Do not directly close the file pointer by `fclose` or other functions.
Instead you should close `IOHandle` by `IO::close_file`.

NOTE:
If `IO::close` is called while using the `Ptr` obtained by this function, the `Ptr` becomes invalid and may cause undefined behavior.

### namespace Std::Iterator

#### advance

Type: `[iter : Std::Iterator] iter -> Std::Option (iter, Std::Iterator::Item iter)`

#### append

Type: `[i1 : Std::Iterator, i2 : Std::Iterator, Std::Iterator::Item i1 = a, Std::Iterator::Item i2 = a] i2 -> i1 -> Std::Iterator::AppendIterator i1 i2`

Append two iterators.

NOTE: Since this function is designed so that `iter1.append(iter2)` appends `iter2` after `iter1`, `append(iter1, iter2)` appends iterators in the opposite order.

#### bang

Type: `[iter : Std::Iterator, Std::Iterator::Item iter = a] iter -> Std::Iterator::ArrayIterator a`

Convert any iterator to an array iterator.

All elements of the input iterator are collected into an array. Therefore, this function may consume a lot of memory.
On the other hand, iteration may be faster by banging.

#### collect_m

Type: `[m : Std::Monad, iter : Std::Iterator, Std::Iterator::Item iter = m a] iter -> m (Std::Array a)`

Executes monadic actions and collects the results into an array.

#### count_up

Type: `Std::I64 -> Std::Iterator::CountUpIterator`

Create an iterator that counts up from a number.

`count_up(start)` generates an infinite sequence of numbers starting from `start`.

#### empty

Type: `Std::Iterator::EmptyIterator a`

An iterator that yields no elements.

NOTE: When using this iterator, you may need to specify the type of the iterator explicitly, e.g, `(empty : EmptyIterator I64)`.

#### filter

Type: `[i : Std::Iterator, Std::Iterator::Item i = a] (a -> Std::Bool) -> i -> Std::Iterator::FilterIterator i a`

Filter the elements of an iterator by a predicate.

`iter.filter(pred)` returns an iterator that only yields elements of `iter` for which `pred` returns `true`.

#### filter_map

Type: `[i : Std::Iterator, Std::Iterator::Item i = a] (a -> Std::Option b) -> i -> Std::Iterator::FilterMapIterator i a b`

Filter and map the elements of an iterator.

`iter.filter_map(f)` returns an iterator that applies `f` to each element of `iter` and yields the result if it is `some`.

#### flat_map

Type: `[i1 : Std::Iterator, i2 : Std::Iterator, Std::Iterator::Item i1 = a, Std::Iterator::Item i2 = b] (a -> i2) -> i1 -> Std::Iterator::FlatMapIterator i1 a i2`

#### flatten

Type: `[i2 : Std::Iterator, i1 : Std::Iterator, Std::Iterator::Item i2 = i1] i2 -> Std::Iterator::FlattenIterator i2 i1`

Flatten an iterator of iterators.

#### fold

Type: `[iter : Std::Iterator, Std::Iterator::Item iter = a] s -> (a -> s -> s) -> iter -> s`

Fold the elements of an iterator from left to right.

Conceptually, `[a0, a1, a2, ...].to_iter.fold(s, op) = s.op(a0).op(a1).op(a2)...`.

#### fold_m

Type: `[m : Std::Monad, iter : Std::Iterator, Std::Iterator::Item iter = a] s -> (a -> s -> m s) -> iter -> m s`

Fold the elements of an iterator from left to right by monadic action.

#### from_array

Type: `Std::Array a -> Std::Iterator::ArrayIterator a`

Create an iterator from an array.

#### from_map

Type: `(Std::I64 -> a) -> Std::Iterator::MapIterator Std::Iterator::CountUpIterator Std::I64 a`

Create an iterator by a function that returns element at each index.

#### generate

Type: `s -> (s -> Std::Option (s, a)) -> Std::Iterator::StateIterator s a`

Create an iterator that generates elements by the state transition function.

#### get_first

Type: `[iter : Std::Iterator] iter -> Std::Option (Std::Iterator::Item iter)`

Get the first element of an iterator.

If the iterator is empty, this function returns `none`.

#### get_size

Type: `[iter : Std::Iterator] iter -> Std::I64`

Get the number of elements in an iterator.

#### get_tail

Type: `[iter : Std::Iterator] iter -> Std::Option iter`

Get the tail of an iterator.

If the iterator is empty, this function returns `none`.

#### intersperse

Type: `[i : Std::Iterator, Std::Iterator::Item i = a] a -> i -> Std::Iterator::IntersperseIterator i a`

Intersperse an element between elements of an iterator.

Example:
```
assert_eq(|_|"", [1, 2, 3].from_array.intersperse(0).to_array, [1, 0, 2, 0, 3]);;
```

#### is_empty

Type: `[iter : Std::Iterator] iter -> Std::Bool`

Is an iterator empty?

#### is_equal

Type: `[iter1 : Std::Iterator, iter2 : Std::Iterator, a : Std::Eq, Std::Iterator::Item iter1 = a, Std::Iterator::Item iter2 = a] iter1 -> iter2 -> Std::Bool`

Compare two iterators by their elements.

#### loop_iter

Type: `[iter : Std::Iterator, Std::Iterator::Item iter = a] s -> (a -> s -> Std::LoopState s s) -> iter -> s`

Loop over the elements of an iterator.

This function is similar to `fold` but a more general version of it. It allows the user to break out of the loop at any point.

#### loop_iter_m

Type: `[m : Std::Monad, iter : Std::Iterator, Std::Iterator::Item iter = a] s -> (a -> s -> m (Std::LoopState s s)) -> iter -> m s`

Loop over the elements of an iterator by monadic action.

#### map

Type: `[i : Std::Iterator, Std::Iterator::Item i = a] (a -> b) -> i -> Std::Iterator::MapIterator i a b`

Map a function over an iterator.

`iter.map(f)` returns an iterator that applies `f` to each element of `iter`.

#### pop_first

Type: `[iter : Std::Iterator] iter -> iter`

Remove the first element of an iterator.

If the iterator is empty, this function does nothing.

#### product

Type: `[i1 : Std::Iterator, i2 : Std::Iterator, Std::Iterator::Item i1 = a, Std::Iterator::Item i2 = b] i2 -> i1 -> Std::Iterator::ProductIterator i1 i2 a b`

Create an iterator that yields the Cartesian product of two iterators.

NOTE: Since this function is designed so that `iter1.product(iter2)` yields the Cartesian product, the elements of `product(iter2, iter1)` are in the opposite order.

Example:
```
assert_eq(|_|"", range(1, 4).product(['a', 'b'].from_array).to_array, [(1, 'a'), (2, 'a'), (3, 'a'), (1, 'b'), (2, 'b'), (3, 'b')]);;
```

#### push_front

Type: `[i : Std::Iterator, Std::Iterator::Item i = a] a -> i -> Std::Iterator::ConsIterator i a`

Push an element to an iterator.

#### range

Type: `Std::I64 -> Std::I64 -> Std::Iterator::RangeIterator`

Create an iterator that generates a range of numbers.

`range(a, b)` generates a range of numbers from `a` to `b - 1`.

If `a` is greater than or equal to `b`, the iterator will an infinite sequence of `a`.

#### range_step

Type: `Std::I64 -> Std::I64 -> Std::I64 -> Std::Iterator::RangeStepIterator`

Create an iterator that generates a range of numbers with a step.

#### reverse

Type: `[i : Std::Iterator, Std::Iterator::Item i = a] i -> Std::Iterator::ReverseIterator i a`

Reverses an iterator.

NOTE: This function puts all elements of the iterator into an array, so it may consume a lot of memory.

#### sum

Type: `[iter : Std::Iterator, a : Std::Additive, Std::Iterator::Item iter = a] iter -> a`

Calcculate sum of the elements of an iterator.

#### take

Type: `[i : Std::Iterator] Std::I64 -> i -> Std::Iterator::TakeIterator i`

Take the first `n` elements of an iterator.

#### take_while

Type: `[i : Std::Iterator, Std::Iterator::Item i = a] (a -> Std::Bool) -> i -> Std::Iterator::TakeWhileIterator i a`

Take elements from an iterator while a predicate holds.

#### to_array

Type: `[iter : Std::Iterator, Std::Iterator::Item iter = a] iter -> Std::Array a`

Convert an iterator to an array.

#### to_dyn

Type: `[iter : Std::Iterator, Std::Iterator::Item iter = a] iter -> Std::Iterator::DynIterator a`

Convert an iterator into a dynamic iterator.

#### zip

Type: `[i1 : Std::Iterator, i2 : Std::Iterator] i2 -> i1 -> Std::Iterator::ZipIterator i1 i2`

Zip two iterators.

NOTE: Since this function is designed so that `iter1.zip(iter2)` zips `iter1` and `iter2`, the elements of `zip(iter2, iter1)` are in the opposite order.

### namespace Std::Iterator::DynIterator

#### empty

Type: `Std::Iterator::DynIterator a`

Creates an empty dynamic iterator.

### namespace Std::LessThan

#### less_than

Type: `[a : Std::LessThan] a -> a -> Std::Bool`

Compares two values. An expression `x < y` is translated to `less_than(x, y)`.

#### max

Type: `[a : Std::LessThan] a -> a -> a`

#### min

Type: `[a : Std::LessThan] a -> a -> a`

### namespace Std::LessThanOrEq

#### less_than_or_eq

Type: `[a : Std::LessThanOrEq] a -> a -> Std::Bool`

Compares two values. An expression `x <= y` is translated to `less_than_or_eq(x, y)`.

### namespace Std::LoopState

#### break_m

Type: `[m : Std::Monad] r -> m (Std::LoopState s r)`

Make a break value wrapped in a monad.

This is used with `loop_m` function.

#### continue_m

Type: `[m : Std::Monad] s -> m (Std::LoopState s r)`

Make a continue value wrapped in a monad.

This is used with `loop_m` function.

### namespace Std::Monad

#### bind

Type: `[m : Std::Monad] (a -> m b) -> m a -> m b`

#### flatten

Type: `[m : Std::Monad] m (m a) -> m a`

Flattens a nested monadic action.

#### pure

Type: `[m : Std::Monad] a -> m a`

#### unless

Type: `[m : Std::Monad] Std::Bool -> m () -> m ()`

`unless(cond, act)` where `act` is a monadic value which returns `()` perfoms `act` only when `cond` is false.

#### when

Type: `[m : Std::Monad] Std::Bool -> m () -> m ()`

`when(cond, act)` where `act` is a monadic value which returns `()` perfoms `act` only when `cond` is true.

### namespace Std::Mul

#### mul

Type: `[a : Std::Mul] a -> a -> a`

Multiplies a value by another value. An expression `x * y` is translated to `mul(x, y)`.

### namespace Std::Neg

#### neg

Type: `[a : Std::Neg] a -> a`

Negates a value. An expression `-x` is translated to `neg(x)`.

### namespace Std::Not

#### not

Type: `[a : Std::Not] a -> a`

Logical NOT of a value. An expression `!x` is translated to `not(x)`.

### namespace Std::Option

#### as_some_or

Type: `a -> Std::Option a -> a`

Unwrap an option value if it is `some`, or returns given default value if it is `none`.

#### map_or

Type: `b -> (a -> b) -> Std::Option a -> b`

Returns the provided default value if the option is none, or applies a function to the contained value if the option is some.

#### to_iter

Type: `Std::Option a -> Std::Option::OptionIterator (Std::Option a)`

Converts an option into an iterator.

### namespace Std::Ptr

#### add_offset

Type: `Std::I64 -> Std::Ptr -> Std::Ptr`

Adds an offset to a pointer.

#### subtract_ptr

Type: `Std::Ptr -> Std::Ptr -> Std::I64`

Subtracts two pointers.

Note that `x.subtract_ptr(y)` calculates `x - y`, so `subtract_ptr(x, y)` calculates `y - x`.

### namespace Std::PunchedArray

#### plug_in

Type: `a -> Std::PunchedArray a -> Std::Array a`

Plug in an element to a punched array to get back an array.

#### unsafe_punch

Type: `Std::I64 -> Std::Array a -> (Std::PunchedArray a, a)`

Creates a punched array by moving out the element at the specified index.

NOTE: this function assumes that the given array is unique WITHOUT CHECKING.
The uniqueness of the array is ensured in the `Array::act` function.

### namespace Std::Rem

#### rem

Type: `[a : Std::Rem] a -> a -> a`

Calculate remainder of a value dividing another value. An expression `x % y` is translated to `rem(x, y)`.

### namespace Std::Result

#### unwrap

Type: `Std::Result e o -> o`

Returns the containing value if the value is ok, or otherwise aborts the program.

### namespace Std::String

#### _get_c_str

Type: `Std::String -> Std::Ptr`

Get the null-terminated C string.

Note that in case the string is not used after call of this function, the returned pointer will be already released.

#### _unsafe_from_c_str

Type: `Std::Array Std::U8 -> Std::String`

Create a string from C string (i.e., null-terminated byte array).

If the byte array doesn't include `\0`, this function causes undefined behavior.

#### _unsafe_from_c_str_ptr

Type: `Std::Ptr -> Std::String`

Create a `String` from a pointer to null-terminated C string.

If `ptr` is not pointing to a valid null-terminated C string, this function cause undefined behavior.

#### borrow_c_str

Type: `(Std::Ptr -> a) -> Std::String -> a`

Call a function with a null-terminated C string.

#### borrow_c_str_io

Type: `(Std::Ptr -> Std::IO a) -> Std::String -> Std::IO a`

Call an IO action with a null-terminated C string.

#### concat

Type: `Std::String -> Std::String -> Std::String`

Concatenate two strings.

Note: Since `s1.concat(s2)` puts `s2` after `s1`, `concat(lhs, rhs)` puts `lhs` after `rhs`.

#### concat_iter

Type: `[strs : Std::Iterator, Std::Iterator::Item strs = Std::String] strs -> Std::String`

Concatenate an iterator of strings.

#### empty

Type: `Std::I64 -> Std::String`

Create an empty string, which is reserved for a length.

#### find

Type: `Std::String -> Std::I64 -> Std::String -> Std::Option Std::I64`

`str.find(token, start_idx)` finds the index where `token` firstly appears in `str`, starting from `start_idx`.

Note that this function basically returns a number less than or equal to `start_idx`, but there is an exception:
`str.find("", start_idx)` with `start_idx >= str.get_size` returns `str.get_size`, not `start_idx`.

#### from_U8

Type: `Std::U8 -> Std::String`

Creates a string from a byte.

Example:
```
assert_eq(|_|"", String::from_U8('a'), "a");;
assert_eq(|_|"", String::from_U8('\x00'), "");;
```

#### get_bytes

Type: `Std::String -> Std::Array Std::U8`

Gets the byte array of a string, containing null-terminator.

#### get_first_byte

Type: `Std::String -> Std::Option Std::U8`

Gets the first byte of a string. Returns none if the string is empty.

#### get_last_byte

Type: `Std::String -> Std::Option Std::U8`

Gets the last byte of a string. Returns none if the string is empty.

#### get_size

Type: `Std::String -> Std::I64`

Gets the length of a string.

#### get_sub

Type: `Std::I64 -> Std::I64 -> Std::String -> Std::String`

`String` version of `Array::get_sub`.

#### is_empty

Type: `Std::String -> Std::Bool`

Returns if the string is empty or not.

#### join

Type: `[ss : Std::Iterator, Std::Iterator::Item ss = Std::String] Std::String -> ss -> Std::String`

Joins (an iterator of) strings by a separator.

#### pop_back_byte

Type: `Std::String -> Std::String`

Removes the last byte.

If the string is empty, this function does nothing.

#### split

Type: `Std::String -> Std::String -> Std::String::StringSplitIterator`

`str.split(sep)` splits `str` by `sep` into an iterator.

Example:
```
assert_eq(|_|"Ex. 1", "ab,c,".split(",").to_array, ["ab", "c", ""]);;
assert_eq(|_|"Ex. 2", "abc".split(",").to_array, ["abc"]);;
assert_eq(|_|"Ex. 3", "abc".split("").to_array, ["a", "b", "c"]);; // Special behavior when the separator is empty.
```

#### strip_first_bytes

Type: `(Std::U8 -> Std::Bool) -> Std::String -> Std::String`

Removes the first byte of a string while it satisifies the specified condition.

#### strip_first_spaces

Type: `Std::String -> Std::String`

Removes leading whitespace characters.

#### strip_last_bytes

Type: `(Std::U8 -> Std::Bool) -> Std::String -> Std::String`

Removes the last byte of a string while it satisifies the specified condition.

#### strip_last_newlines

Type: `Std::String -> Std::String`

Removes newlines and carriage returns at the end of the string.

#### strip_last_spaces

Type: `Std::String -> Std::String`

Removes trailing whitespace characters.

#### strip_spaces

Type: `Std::String -> Std::String`

Strips leading and trailing whitespace characters.

### namespace Std::Sub

#### sub

Type: `[a : Std::Sub] a -> a -> a`

Subtracts a value from another value. An expression `x - y` is translated to `sub(x, y)`.

### namespace Std::ToBytes

#### to_bytes

Type: `[a : Std::ToBytes] a -> Std::Array Std::U8`

### namespace Std::ToString

#### to_string

Type: `[a : Std::ToString] a -> Std::String`

### namespace Std::U16

#### bit_and

Type: `Std::U16 -> Std::U16 -> Std::U16`

Calculates bitwise AND of two values.

#### bit_or

Type: `Std::U16 -> Std::U16 -> Std::U16`

Calculates bitwise OR of two values.

#### bit_xor

Type: `Std::U16 -> Std::U16 -> Std::U16`

Calculates bitwise XOR of two values.

#### maximum

Type: `Std::U16`

#### minimum

Type: `Std::U16`

#### shift_left

Type: `Std::U16 -> Std::U16 -> Std::U16`

`v.shift_left(w)` shifts `v` to left by `w` bits.

#### shift_right

Type: `Std::U16 -> Std::U16 -> Std::U16`

`v.shift_right(w)` shifts `v` to right by `w` bits.

#### to_CChar

Type: `Std::U16 -> Std::I8`

Casts a value of `U16` into a value of `CChar`.

#### to_CDouble

Type: `Std::U16 -> Std::F64`

Casts a value of `U16` into a value of `CDouble`.

#### to_CFloat

Type: `Std::U16 -> Std::F32`

Casts a value of `U16` into a value of `CFloat`.

#### to_CInt

Type: `Std::U16 -> Std::I32`

Casts a value of `U16` into a value of `CInt`.

#### to_CLong

Type: `Std::U16 -> Std::I64`

Casts a value of `U16` into a value of `CLong`.

#### to_CLongLong

Type: `Std::U16 -> Std::I64`

Casts a value of `U16` into a value of `CLongLong`.

#### to_CShort

Type: `Std::U16 -> Std::I16`

Casts a value of `U16` into a value of `CShort`.

#### to_CSizeT

Type: `Std::U16 -> Std::U64`

Casts a value of `U16` into a value of `CSizeT`.

#### to_CUnsignedChar

Type: `Std::U16 -> Std::U8`

Casts a value of `U16` into a value of `CUnsignedChar`.

#### to_CUnsignedInt

Type: `Std::U16 -> Std::U32`

Casts a value of `U16` into a value of `CUnsignedInt`.

#### to_CUnsignedLong

Type: `Std::U16 -> Std::U64`

Casts a value of `U16` into a value of `CUnsignedLong`.

#### to_CUnsignedLongLong

Type: `Std::U16 -> Std::U64`

Casts a value of `U16` into a value of `CUnsignedLongLong`.

#### to_CUnsignedShort

Type: `Std::U16 -> Std::U16`

Casts a value of `U16` into a value of `CUnsignedShort`.

#### to_F32

Type: `Std::U16 -> Std::F32`

Casts a value of `U16` into a value of `F32`.

#### to_F64

Type: `Std::U16 -> Std::F64`

Casts a value of `U16` into a value of `F64`.

#### to_I16

Type: `Std::U16 -> Std::I16`

Casts a value of `U16` into a value of `I16`.

#### to_I32

Type: `Std::U16 -> Std::I32`

Casts a value of `U16` into a value of `I32`.

#### to_I64

Type: `Std::U16 -> Std::I64`

Casts a value of `U16` into a value of `I64`.

#### to_I8

Type: `Std::U16 -> Std::I8`

Casts a value of `U16` into a value of `I8`.

#### to_U16

Type: `Std::U16 -> Std::U16`

Casts a value of `U16` into a value of `U16`.

#### to_U32

Type: `Std::U16 -> Std::U32`

Casts a value of `U16` into a value of `U32`.

#### to_U64

Type: `Std::U16 -> Std::U64`

Casts a value of `U16` into a value of `U64`.

#### to_U8

Type: `Std::U16 -> Std::U8`

Casts a value of `U16` into a value of `U8`.

### namespace Std::U32

#### bit_and

Type: `Std::U32 -> Std::U32 -> Std::U32`

Calculates bitwise AND of two values.

#### bit_or

Type: `Std::U32 -> Std::U32 -> Std::U32`

Calculates bitwise OR of two values.

#### bit_xor

Type: `Std::U32 -> Std::U32 -> Std::U32`

Calculates bitwise XOR of two values.

#### maximum

Type: `Std::U32`

#### minimum

Type: `Std::U32`

#### shift_left

Type: `Std::U32 -> Std::U32 -> Std::U32`

`v.shift_left(w)` shifts `v` to left by `w` bits.

#### shift_right

Type: `Std::U32 -> Std::U32 -> Std::U32`

`v.shift_right(w)` shifts `v` to right by `w` bits.

#### to_CChar

Type: `Std::U32 -> Std::I8`

Casts a value of `U32` into a value of `CChar`.

#### to_CDouble

Type: `Std::U32 -> Std::F64`

Casts a value of `U32` into a value of `CDouble`.

#### to_CFloat

Type: `Std::U32 -> Std::F32`

Casts a value of `U32` into a value of `CFloat`.

#### to_CInt

Type: `Std::U32 -> Std::I32`

Casts a value of `U32` into a value of `CInt`.

#### to_CLong

Type: `Std::U32 -> Std::I64`

Casts a value of `U32` into a value of `CLong`.

#### to_CLongLong

Type: `Std::U32 -> Std::I64`

Casts a value of `U32` into a value of `CLongLong`.

#### to_CShort

Type: `Std::U32 -> Std::I16`

Casts a value of `U32` into a value of `CShort`.

#### to_CSizeT

Type: `Std::U32 -> Std::U64`

Casts a value of `U32` into a value of `CSizeT`.

#### to_CUnsignedChar

Type: `Std::U32 -> Std::U8`

Casts a value of `U32` into a value of `CUnsignedChar`.

#### to_CUnsignedInt

Type: `Std::U32 -> Std::U32`

Casts a value of `U32` into a value of `CUnsignedInt`.

#### to_CUnsignedLong

Type: `Std::U32 -> Std::U64`

Casts a value of `U32` into a value of `CUnsignedLong`.

#### to_CUnsignedLongLong

Type: `Std::U32 -> Std::U64`

Casts a value of `U32` into a value of `CUnsignedLongLong`.

#### to_CUnsignedShort

Type: `Std::U32 -> Std::U16`

Casts a value of `U32` into a value of `CUnsignedShort`.

#### to_F32

Type: `Std::U32 -> Std::F32`

Casts a value of `U32` into a value of `F32`.

#### to_F64

Type: `Std::U32 -> Std::F64`

Casts a value of `U32` into a value of `F64`.

#### to_I16

Type: `Std::U32 -> Std::I16`

Casts a value of `U32` into a value of `I16`.

#### to_I32

Type: `Std::U32 -> Std::I32`

Casts a value of `U32` into a value of `I32`.

#### to_I64

Type: `Std::U32 -> Std::I64`

Casts a value of `U32` into a value of `I64`.

#### to_I8

Type: `Std::U32 -> Std::I8`

Casts a value of `U32` into a value of `I8`.

#### to_U16

Type: `Std::U32 -> Std::U16`

Casts a value of `U32` into a value of `U16`.

#### to_U32

Type: `Std::U32 -> Std::U32`

Casts a value of `U32` into a value of `U32`.

#### to_U64

Type: `Std::U32 -> Std::U64`

Casts a value of `U32` into a value of `U64`.

#### to_U8

Type: `Std::U32 -> Std::U8`

Casts a value of `U32` into a value of `U8`.

### namespace Std::U64

#### bit_and

Type: `Std::U64 -> Std::U64 -> Std::U64`

Calculates bitwise AND of two values.

#### bit_or

Type: `Std::U64 -> Std::U64 -> Std::U64`

Calculates bitwise OR of two values.

#### bit_xor

Type: `Std::U64 -> Std::U64 -> Std::U64`

Calculates bitwise XOR of two values.

#### maximum

Type: `Std::U64`

#### minimum

Type: `Std::U64`

#### shift_left

Type: `Std::U64 -> Std::U64 -> Std::U64`

`v.shift_left(w)` shifts `v` to left by `w` bits.

#### shift_right

Type: `Std::U64 -> Std::U64 -> Std::U64`

`v.shift_right(w)` shifts `v` to right by `w` bits.

#### to_CChar

Type: `Std::U64 -> Std::I8`

Casts a value of `U64` into a value of `CChar`.

#### to_CDouble

Type: `Std::U64 -> Std::F64`

Casts a value of `U64` into a value of `CDouble`.

#### to_CFloat

Type: `Std::U64 -> Std::F32`

Casts a value of `U64` into a value of `CFloat`.

#### to_CInt

Type: `Std::U64 -> Std::I32`

Casts a value of `U64` into a value of `CInt`.

#### to_CLong

Type: `Std::U64 -> Std::I64`

Casts a value of `U64` into a value of `CLong`.

#### to_CLongLong

Type: `Std::U64 -> Std::I64`

Casts a value of `U64` into a value of `CLongLong`.

#### to_CShort

Type: `Std::U64 -> Std::I16`

Casts a value of `U64` into a value of `CShort`.

#### to_CSizeT

Type: `Std::U64 -> Std::U64`

Casts a value of `U64` into a value of `CSizeT`.

#### to_CUnsignedChar

Type: `Std::U64 -> Std::U8`

Casts a value of `U64` into a value of `CUnsignedChar`.

#### to_CUnsignedInt

Type: `Std::U64 -> Std::U32`

Casts a value of `U64` into a value of `CUnsignedInt`.

#### to_CUnsignedLong

Type: `Std::U64 -> Std::U64`

Casts a value of `U64` into a value of `CUnsignedLong`.

#### to_CUnsignedLongLong

Type: `Std::U64 -> Std::U64`

Casts a value of `U64` into a value of `CUnsignedLongLong`.

#### to_CUnsignedShort

Type: `Std::U64 -> Std::U16`

Casts a value of `U64` into a value of `CUnsignedShort`.

#### to_F32

Type: `Std::U64 -> Std::F32`

Casts a value of `U64` into a value of `F32`.

#### to_F64

Type: `Std::U64 -> Std::F64`

Casts a value of `U64` into a value of `F64`.

#### to_I16

Type: `Std::U64 -> Std::I16`

Casts a value of `U64` into a value of `I16`.

#### to_I32

Type: `Std::U64 -> Std::I32`

Casts a value of `U64` into a value of `I32`.

#### to_I64

Type: `Std::U64 -> Std::I64`

Casts a value of `U64` into a value of `I64`.

#### to_I8

Type: `Std::U64 -> Std::I8`

Casts a value of `U64` into a value of `I8`.

#### to_U16

Type: `Std::U64 -> Std::U16`

Casts a value of `U64` into a value of `U16`.

#### to_U32

Type: `Std::U64 -> Std::U32`

Casts a value of `U64` into a value of `U32`.

#### to_U64

Type: `Std::U64 -> Std::U64`

Casts a value of `U64` into a value of `U64`.

#### to_U8

Type: `Std::U64 -> Std::U8`

Casts a value of `U64` into a value of `U8`.

### namespace Std::U8

#### bit_and

Type: `Std::U8 -> Std::U8 -> Std::U8`

Calculates bitwise AND of two values.

#### bit_or

Type: `Std::U8 -> Std::U8 -> Std::U8`

Calculates bitwise OR of two values.

#### bit_xor

Type: `Std::U8 -> Std::U8 -> Std::U8`

Calculates bitwise XOR of two values.

#### maximum

Type: `Std::U8`

#### minimum

Type: `Std::U8`

#### shift_left

Type: `Std::U8 -> Std::U8 -> Std::U8`

`v.shift_left(w)` shifts `v` to left by `w` bits.

#### shift_right

Type: `Std::U8 -> Std::U8 -> Std::U8`

`v.shift_right(w)` shifts `v` to right by `w` bits.

#### to_CChar

Type: `Std::U8 -> Std::I8`

Casts a value of `U8` into a value of `CChar`.

#### to_CDouble

Type: `Std::U8 -> Std::F64`

Casts a value of `U8` into a value of `CDouble`.

#### to_CFloat

Type: `Std::U8 -> Std::F32`

Casts a value of `U8` into a value of `CFloat`.

#### to_CInt

Type: `Std::U8 -> Std::I32`

Casts a value of `U8` into a value of `CInt`.

#### to_CLong

Type: `Std::U8 -> Std::I64`

Casts a value of `U8` into a value of `CLong`.

#### to_CLongLong

Type: `Std::U8 -> Std::I64`

Casts a value of `U8` into a value of `CLongLong`.

#### to_CShort

Type: `Std::U8 -> Std::I16`

Casts a value of `U8` into a value of `CShort`.

#### to_CSizeT

Type: `Std::U8 -> Std::U64`

Casts a value of `U8` into a value of `CSizeT`.

#### to_CUnsignedChar

Type: `Std::U8 -> Std::U8`

Casts a value of `U8` into a value of `CUnsignedChar`.

#### to_CUnsignedInt

Type: `Std::U8 -> Std::U32`

Casts a value of `U8` into a value of `CUnsignedInt`.

#### to_CUnsignedLong

Type: `Std::U8 -> Std::U64`

Casts a value of `U8` into a value of `CUnsignedLong`.

#### to_CUnsignedLongLong

Type: `Std::U8 -> Std::U64`

Casts a value of `U8` into a value of `CUnsignedLongLong`.

#### to_CUnsignedShort

Type: `Std::U8 -> Std::U16`

Casts a value of `U8` into a value of `CUnsignedShort`.

#### to_F32

Type: `Std::U8 -> Std::F32`

Casts a value of `U8` into a value of `F32`.

#### to_F64

Type: `Std::U8 -> Std::F64`

Casts a value of `U8` into a value of `F64`.

#### to_I16

Type: `Std::U8 -> Std::I16`

Casts a value of `U8` into a value of `I16`.

#### to_I32

Type: `Std::U8 -> Std::I32`

Casts a value of `U8` into a value of `I32`.

#### to_I64

Type: `Std::U8 -> Std::I64`

Casts a value of `U8` into a value of `I64`.

#### to_I8

Type: `Std::U8 -> Std::I8`

Casts a value of `U8` into a value of `I8`.

#### to_U16

Type: `Std::U8 -> Std::U16`

Casts a value of `U8` into a value of `U16`.

#### to_U32

Type: `Std::U8 -> Std::U32`

Casts a value of `U8` into a value of `U32`.

#### to_U64

Type: `Std::U8 -> Std::U64`

Casts a value of `U8` into a value of `U64`.

#### to_U8

Type: `Std::U8 -> Std::U8`

Casts a value of `U8` into a value of `U8`.

### namespace Std::Zero

#### zero

Type: `[a : Std::Zero] a`

## Types and aliases

### namespace Std

#### Array

Defined as: `type Array a = box { built-in }`

The type of variable length arrays. This is a boxed type.

#### Arrow

Defined as: `type Arrow a b = unbox { built-in }`

`Arrow a b` represents the type of a function that takes a value of type `a` and returns a value of type `b`. Usually written as `a -> b`.

#### Bool

Defined as: `type Bool = unbox { built-in }`

The type of boolean values.

#### Box

Defined as: `type Box a = box struct { ...fields... }`

Boxed wrapper for a type.

##### field `value`

Type: `a`

#### ErrMsg

Defined as: `type ErrMsg = Std::String`

A type (alias) for error message.

#### F32

Defined as: `type F32 = unbox { built-in }`

The type of 32-bit floating point values.

#### F64

Defined as: `type F64 = unbox { built-in }`

The type of 64-bit floating point values.

#### I16

Defined as: `type I16 = unbox { built-in }`

The type of 16-bit signed integers.

#### I32

Defined as: `type I32 = unbox { built-in }`

The type of 32-bit signed integers.

#### I64

Defined as: `type I64 = unbox { built-in }`

The type of 64-bit signed integers.

#### I8

Defined as: `type I8 = unbox { built-in }`

The type of 8-bit signed integers.

#### IO

Defined as: `type IO a = unbox struct { ...fields... }`

`IO a` is a type representing I/O actions which return values of type `a`.

##### field `runner`

Type: `Std::IO::IOState -> (Std::IO::IOState, a)`

#### Lazy

Defined as: `type Lazy = () -> a`

The type of lazily generated values.

You can create a lazy value by `|_| (...an expression to generate the value...)`,
and you can evaluate a lazy value `v` by `v()`.

#### LoopState

Defined as: `type LoopState s r = unbox union { ...variants... }`

A union type with variants `continue` and `break`.

This type is used to represent the result of a loop body function passed to `Std::loop` or other similar functions.

##### variant `continue`

Type: `s`

##### variant `break`

Type: `r`

#### Option

Defined as: `type Option a = unbox union { ...variants... }`

##### variant `none`

Type: `()`

##### variant `some`

Type: `a`

#### Path

Defined as: `type Path = Std::String`

The type for file path.

#### Ptr

Defined as: `type Ptr = unbox { built-in }`

The type of pointers.

#### PunchedArray

Defined as: `type PunchedArray a = unbox struct { ...fields... }`

The type of punched arrays.

A punched array is an array from which a certain element has been removed.
This is used in the implementation of `Array::act`.

##### field `_arr`

Type: `Std::Array a`

##### field `idx`

Type: `Std::I64`

#### Result

Defined as: `type Result e o = unbox union { ...variants... }`

A type of result value for a computation that may fail.

##### variant `ok`

Type: `o`

##### variant `err`

Type: `e`

#### String

Defined as: `type String = unbox struct { ...fields... }`

##### field `_data`

Type: `Std::Array Std::U8`

#### Tuple0

Defined as: `type Tuple0 = unbox struct { ...fields... }`

#### Tuple2

Defined as: `type Tuple2 t0 t1 = unbox struct { ...fields... }`

##### field `0`

Type: `t0`

##### field `1`

Type: `t1`

#### Tuple3

Defined as: `type Tuple3 t0 t1 t2 = unbox struct { ...fields... }`

##### field `0`

Type: `t0`

##### field `1`

Type: `t1`

##### field `2`

Type: `t2`

#### U16

Defined as: `type U16 = unbox { built-in }`

The type of 16-bit unsigned integers.

#### U32

Defined as: `type U32 = unbox { built-in }`

The type of 32-bit unsigned integers.

#### U64

Defined as: `type U64 = unbox { built-in }`

The type of 64-bit unsigned integers.

#### U8

Defined as: `type U8 = unbox { built-in }`

The type of 8-bit unsinged integers.

### namespace Std::FFI

#### CChar

Defined as: `type CChar = Std::I8`

#### CDouble

Defined as: `type CDouble = Std::F64`

#### CFloat

Defined as: `type CFloat = Std::F32`

#### CInt

Defined as: `type CInt = Std::I32`

#### CLong

Defined as: `type CLong = Std::I64`

#### CLongLong

Defined as: `type CLongLong = Std::I64`

#### CShort

Defined as: `type CShort = Std::I16`

#### CSizeT

Defined as: `type CSizeT = Std::U64`

#### CUnsignedChar

Defined as: `type CUnsignedChar = Std::U8`

#### CUnsignedInt

Defined as: `type CUnsignedInt = Std::U32`

#### CUnsignedLong

Defined as: `type CUnsignedLong = Std::U64`

#### CUnsignedLongLong

Defined as: `type CUnsignedLongLong = Std::U64`

#### CUnsignedShort

Defined as: `type CUnsignedShort = Std::U16`

#### Destructor

Defined as: `type Destructor a = box struct { ...fields... }`

`Destructor a` is a wrapper type for `a`, which can have a destructor function `a -> IO a`.
Just before a value of type `Destructor a` is dropped, the destructor function is called on the contained value, and the value can be modified by the `IO` action.

This type is used to create a Fix's type that wraps a resource allocated by FFI. In such cases, the destructor release the resource by FFI.

NOTE: In the destructor, only IO actions for finalizing the passed value are allowed, and you should not perform other IO actions such as writing standard output.

NOTE: Of course, if the value stored in `Destructor` also exists outside of `Destructor`, the value still exists in the Fix program even after the destructor function is called,
and there is a possibility that the value is used after the destructor function is called.

##### field `_value`

Type: `a`

##### field `dtor`

Type: `a -> Std::IO a`

### namespace Std::IO

#### IOFail

Defined as: `type IOFail a = unbox struct { ...fields... }`

The type for I/O actions which may fail.

##### field `_data`

Type: `Std::IO (Std::Result Std::ErrMsg a)`

#### IOHandle

Defined as: `type IOHandle = unbox struct { ...fields... }`

A handle type for read / write operations on files, stdin, stdout, stderr.

You can create `IOHandle` value by `IO::open_file`, and close it by `IO::close_file`.
There are also global `IO::IOHandle::stdin`, `IO::IOHandle::stdout`, `IO::IOHandle::stderr`.

`IOHandle` is different from C's `FILE` structure in that it is safe to close it twice.
If you try to get a file pointer by `file_ptr` from a closed `IOHandle`, you will get `nullptr`.

NOTE:
`IOHandle` is implemented by `Destructor`, but the destructor function does not close the file pointer.
(The destructor function only frees the management memory area.)
You should explicitly close the file pointer by `IO::close_file`.

##### field `_data`

Type: `Std::FFI::Destructor Std::Ptr`

#### IOState

Defined as: `type IOState = unbox { built-in }`

The type of the "state"s modified by I/O operations. 

The type `IO a` is isomorphic to `IOState -> (IOState, a)`.

Values of type `IOState` must be used linearly, i.e., each value must be used exactly once and must not be duplicated or discarded.

Values of type `IOState` are generated by the runtime when executing `IO` actions like `main` and passed linearly to various places in the program. At some places, `IOState` values are consumed by `FFI_CALL_IOS` expressions and new `IOState` values are generated. When `IO` actions like `main` finish, they are consumed by the runtime and disappear.

Technically, `IOState` exists to specify the execution of I/O operations to the optimizer in the compiler.

### namespace Std::Iterator

#### AppendIterator

Defined as: `type AppendIterator i1 i2 = unbox struct { ...fields... }`

##### field `iter1`

Type: `Std::Option i1`

##### field `iter2`

Type: `i2`

#### ArrayIterator

Defined as: `type ArrayIterator a = unbox struct { ...fields... }`

Iterators that yields elements of an array.

##### field `arr`

Type: `Std::Array a`

##### field `idx`

Type: `Std::I64`

#### ConsIterator

Defined as: `type ConsIterator i a = unbox struct { ...fields... }`

##### field `head`

Type: `Std::Option a`

##### field `tail`

Type: `i`

#### CountUpIterator

Defined as: `type CountUpIterator = unbox struct { ...fields... }`

##### field `next`

Type: `Std::I64`

#### DynIterator

Defined as: `type DynIterator a = unbox struct { ...fields... }`

The type of dynamic iterators.

`DynIterator` has a field, `next`, which is a function that returns the next element and the next iterator.
Therefore, the process to advance `DynIterator` can be determined dynamically at runtime, not at compile time.

The main advantage of dynamic iterator is that since it has a simple type, `DynIterator a`,
- `DynIterator` can be instances of traits such as `Monad`, `Eq`, etc.
- it is possible to return two dynamic iterators with different constructions depending on the branch.

However, iterating over `DynIterator` are much slower than iterating over other iterators provided in this namespace.
Therefore, if performance is important, you should avoid using `DynIterator`.
In particular, if you iterate over the same `DynIterator` multiple times,
consider converting it to an `ArrayIterator` using `bang` before iterating.

##### field `next`

Type: `() -> Std::Option (Std::Iterator::DynIterator a, a)`

#### EmptyIterator

Defined as: `type EmptyIterator a = unbox struct { ...fields... }`

Iterators that yields no elements.

#### FilterIterator

Defined as: `type FilterIterator i a = unbox struct { ...fields... }`

##### field `iter`

Type: `i`

##### field `pred`

Type: `a -> Std::Bool`

#### FilterMapIterator

Defined as: `type FilterMapIterator i a b = unbox struct { ...fields... }`

##### field `iter`

Type: `i`

##### field `f`

Type: `a -> Std::Option b`

#### FlatMapIterator

Defined as: `type FlatMapIterator = Std::Iterator::FlattenIterator (Std::Iterator::MapIterator i1 a i2) i2`

#### FlattenIterator

Defined as: `type FlattenIterator i2 i1 = unbox struct { ...fields... }`

##### field `i2`

Type: `i2`

##### field `i1`

Type: `Std::Option i1`

#### IntersperseIterator

Defined as: `type IntersperseIterator i a = unbox struct { ...fields... }`

##### field `iter`

Type: `i`

##### field `sep`

Type: `a`

##### field `next_is_sep`

Type: `Std::Bool`

#### MapIterator

Defined as: `type MapIterator i a b = unbox struct { ...fields... }`

##### field `iter`

Type: `i`

##### field `f`

Type: `a -> b`

#### ProductIterator

Defined as: `type ProductIterator i1 i2 a b = unbox struct { ...fields... }`

##### field `iter1`

Type: `i1`

##### field `iter2`

Type: `i2`

##### field `e2`

Type: `Std::Option b`

##### field `iter1_org`

Type: `i1`

#### RangeIterator

Defined as: `type RangeIterator = unbox struct { ...fields... }`

Iterators that yields reversed elements of an iterator.

##### field `next`

Type: `Std::I64`

##### field `end`

Type: `Std::I64`

#### RangeStepIterator

Defined as: `type RangeStepIterator = unbox struct { ...fields... }`

##### field `next`

Type: `Std::I64`

##### field `end`

Type: `Std::I64`

##### field `step`

Type: `Std::I64`

#### ReverseIterator

Defined as: `type ReverseIterator i a = unbox struct { ...fields... }`

##### field `idx`

Type: `Std::I64`

##### field `arr`

Type: `Std::Array a`

#### StateIterator

Defined as: `type StateIterator s a = unbox struct { ...fields... }`

##### field `state`

Type: `Std::Option s`

##### field `transit`

Type: `s -> Std::Option (s, a)`

#### TakeIterator

Defined as: `type TakeIterator i = unbox struct { ...fields... }`

Takes at most `n` elements from an iterator.

##### field `iter`

Type: `i`

##### field `n`

Type: `Std::I64`

#### TakeWhileIterator

Defined as: `type TakeWhileIterator i a = unbox struct { ...fields... }`

##### field `iter`

Type: `i`

##### field `pred`

Type: `a -> Std::Bool`

#### ZipIterator

Defined as: `type ZipIterator i1 i2 = unbox struct { ...fields... }`

##### field `iter1`

Type: `i1`

##### field `iter2`

Type: `i2`

### namespace Std::Option

#### OptionIterator

Defined as: `type OptionIterator opt = unbox struct { ...fields... }`

##### field `opt`

Type: `opt`

### namespace Std::String

#### StringSplitIterator

Defined as: `type StringSplitIterator = unbox struct { ...fields... }`

##### field `idx`

Type: `Std::I64`

##### field `str`

Type: `Std::String`

##### field `strlen`

Type: `Std::I64`

##### field `sep`

Type: `Std::String`

##### field `sep_len`

Type: `Std::I64`

## Traits and aliases

### namespace Std

#### trait `a : Add`

Trait for infix operator `+`.

##### method `add`

Type: `a -> a -> a`

Adds two values. An expression `x + y` is translated to `add(x, y)`.

#### trait `a : Boxed`

Marker trait for boxed types.

This trait is automatically implemented for all boxed types.
Implementing this trait manually is not allowed.

#### trait `a : Div`

Trait for infix operator `/`.

##### method `div`

Type: `a -> a -> a`

Divides a value by another value. An expression `x / y` is translated to `div(x, y)`.

#### trait `a : Eq`

Trait for infix operator `==`.

##### method `eq`

Type: `a -> a -> Std::Bool`

Checks equality of two values. An expression `x == y` is translated to `eq(x, y)`.

#### trait `a : FromBytes`

##### method `from_bytes`

Type: `Std::Array Std::U8 -> Std::Result Std::String a`

#### trait `a : FromString`

##### method `from_string`

Type: `Std::String -> Std::Result Std::String a`

#### trait `[f : *->*] f : Functor`

##### method `map`

Type: `(a -> b) -> f a -> f b`

#### trait `iter : Iterator`

The trait of iterators.

Iterator is a concept of a sequence of elements that can be iterated.
More precisely, an iterator is a type whose data is "the current state" and has a method `advance` which returns the next element and the next state.

##### associated type `Item`

Defined as: `Item iter`

##### method `advance`

Type: `iter -> Std::Option (iter, Std::Iterator::Item iter)`

#### trait `a : LessThan`

Trait for infix operator `<`.

##### method `less_than`

Type: `a -> a -> Std::Bool`

Compares two values. An expression `x < y` is translated to `less_than(x, y)`.

#### trait `a : LessThanOrEq`

Trait for infix operator `<=`.

##### method `less_than_or_eq`

Type: `a -> a -> Std::Bool`

Compares two values. An expression `x <= y` is translated to `less_than_or_eq(x, y)`.

#### trait `[m : *->*] m : Monad`

##### method `bind`

Type: `(a -> m b) -> m a -> m b`

##### method `pure`

Type: `a -> m a`

#### trait `a : Mul`

Trait for infix operator `*`.

##### method `mul`

Type: `a -> a -> a`

Multiplies a value by another value. An expression `x * y` is translated to `mul(x, y)`.

#### trait `a : Neg`

Trait for prefix operator `-`.

##### method `neg`

Type: `a -> a`

Negates a value. An expression `-x` is translated to `neg(x)`.

#### trait `a : Not`

Trait for prefix operator `!`.

##### method `not`

Type: `a -> a`

Logical NOT of a value. An expression `!x` is translated to `not(x)`.

#### trait `a : Rem`

Trait for infix operator `%`.

##### method `rem`

Type: `a -> a -> a`

Calculate remainder of a value dividing another value. An expression `x % y` is translated to `rem(x, y)`.

#### trait `a : Sub`

Trait for infix operator `-`.

##### method `sub`

Type: `a -> a -> a`

Subtracts a value from another value. An expression `x - y` is translated to `sub(x, y)`.

#### trait `a : ToBytes`

##### method `to_bytes`

Type: `a -> Std::Array Std::U8`

#### trait `a : ToString`

##### method `to_string`

Type: `a -> Std::String`

#### trait `a : Zero`

##### method `zero`

Type: `a`

## Trait implementations

### impl `() : Std::Eq`

### impl `() : Std::ToString`

Returns "()".

### impl `[t0 : Std::Eq, t1 : Std::Eq] (t0, t1) : Std::Eq`

### impl `[t0 : Std::Eq, t0 : Std::LessThan, t1 : Std::Eq, t1 : Std::LessThan] (t0, t1) : Std::LessThan`

### impl `[t0 : Std::Eq, t0 : Std::LessThanOrEq, t1 : Std::Eq, t1 : Std::LessThanOrEq] (t0, t1) : Std::LessThanOrEq`

### impl `[t0 : Std::ToString, t1 : Std::ToString] (t0, t1) : Std::ToString`

### impl `[t0 : Std::Eq, t1 : Std::Eq, t2 : Std::Eq] (t0, t1, t2) : Std::Eq`

### impl `[t0 : Std::Eq, t0 : Std::LessThan, t1 : Std::Eq, t1 : Std::LessThan, t2 : Std::Eq, t2 : Std::LessThan] (t0, t1, t2) : Std::LessThan`

### impl `[t0 : Std::Eq, t0 : Std::LessThanOrEq, t1 : Std::Eq, t1 : Std::LessThanOrEq, t2 : Std::Eq, t2 : Std::LessThanOrEq] (t0, t1, t2) : Std::LessThanOrEq`

### impl `[t0 : Std::ToString, t1 : Std::ToString, t2 : Std::ToString] (t0, t1, t2) : Std::ToString`

### impl `Std::Array : Std::Functor`

### impl `Std::Array : Std::Monad`

### impl `Std::Array a : Std::Add`

Concatenates two arrays.

### impl `Std::Array a : Std::Boxed`

### impl `[a : Std::Eq] Std::Array a : Std::Eq`

### impl `[a : Std::Eq, a : Std::LessThan] Std::Array a : Std::LessThan`

`LessThan` implementation for `Array a`.

Compares two arrays by lexicographic order.

### impl `[a : Std::Eq, a : Std::LessThanOrEq] Std::Array a : Std::LessThanOrEq`

`LessThanOrEq` implementation for `Array a`.

Compares two arrays by lexicographic order.

### impl `[a : Std::ToString] Std::Array a : Std::ToString`

### impl `Std::Array a : Std::Zero`

The empty array with zero capacity.

### impl `Std::Arrow a : Std::Functor`

### impl `Std::Arrow a : Std::Monad`

### impl `Std::Bool : Std::Eq`

### impl `Std::Bool : Std::Not`

### impl `Std::Bool : Std::ToString`

### impl `Std::Box a : Std::Boxed`

### impl `Std::F32 : Std::Add`

### impl `Std::F32 : Std::Div`

### impl `Std::F32 : Std::Eq`

### impl `Std::F32 : Std::FromBytes`

### impl `Std::F32 : Std::FromString`

### impl `Std::F32 : Std::LessThan`

### impl `Std::F32 : Std::LessThanOrEq`

### impl `Std::F32 : Std::Mul`

### impl `Std::F32 : Std::Neg`

### impl `Std::F32 : Std::Sub`

### impl `Std::F32 : Std::ToBytes`

### impl `Std::F32 : Std::ToString`

### impl `Std::F32 : Std::Zero`

### impl `Std::F64 : Std::Add`

### impl `Std::F64 : Std::Div`

### impl `Std::F64 : Std::Eq`

### impl `Std::F64 : Std::FromBytes`

### impl `Std::F64 : Std::FromString`

### impl `Std::F64 : Std::LessThan`

### impl `Std::F64 : Std::LessThanOrEq`

### impl `Std::F64 : Std::Mul`

### impl `Std::F64 : Std::Neg`

### impl `Std::F64 : Std::Sub`

### impl `Std::F64 : Std::ToBytes`

### impl `Std::F64 : Std::ToString`

### impl `Std::F64 : Std::Zero`

### impl `Std::FFI::Destructor a : Std::Boxed`

### impl `Std::I16 : Std::Add`

### impl `Std::I16 : Std::Div`

### impl `Std::I16 : Std::Eq`

### impl `Std::I16 : Std::FromBytes`

### impl `Std::I16 : Std::FromString`

### impl `Std::I16 : Std::LessThan`

### impl `Std::I16 : Std::LessThanOrEq`

### impl `Std::I16 : Std::Mul`

### impl `Std::I16 : Std::Neg`

### impl `Std::I16 : Std::Rem`

### impl `Std::I16 : Std::Sub`

### impl `Std::I16 : Std::ToBytes`

### impl `Std::I16 : Std::ToString`

### impl `Std::I16 : Std::Zero`

### impl `Std::I32 : Std::Add`

### impl `Std::I32 : Std::Div`

### impl `Std::I32 : Std::Eq`

### impl `Std::I32 : Std::FromBytes`

### impl `Std::I32 : Std::FromString`

### impl `Std::I32 : Std::LessThan`

### impl `Std::I32 : Std::LessThanOrEq`

### impl `Std::I32 : Std::Mul`

### impl `Std::I32 : Std::Neg`

### impl `Std::I32 : Std::Rem`

### impl `Std::I32 : Std::Sub`

### impl `Std::I32 : Std::ToBytes`

### impl `Std::I32 : Std::ToString`

### impl `Std::I32 : Std::Zero`

### impl `Std::I64 : Std::Add`

### impl `Std::I64 : Std::Div`

### impl `Std::I64 : Std::Eq`

### impl `Std::I64 : Std::FromBytes`

### impl `Std::I64 : Std::FromString`

### impl `Std::I64 : Std::LessThan`

### impl `Std::I64 : Std::LessThanOrEq`

### impl `Std::I64 : Std::Mul`

### impl `Std::I64 : Std::Neg`

### impl `Std::I64 : Std::Rem`

### impl `Std::I64 : Std::Sub`

### impl `Std::I64 : Std::ToBytes`

### impl `Std::I64 : Std::ToString`

### impl `Std::I64 : Std::Zero`

### impl `Std::I8 : Std::Add`

### impl `Std::I8 : Std::Div`

### impl `Std::I8 : Std::Eq`

### impl `Std::I8 : Std::FromBytes`

### impl `Std::I8 : Std::FromString`

### impl `Std::I8 : Std::LessThan`

### impl `Std::I8 : Std::LessThanOrEq`

### impl `Std::I8 : Std::Mul`

### impl `Std::I8 : Std::Neg`

### impl `Std::I8 : Std::Rem`

### impl `Std::I8 : Std::Sub`

### impl `Std::I8 : Std::ToBytes`

### impl `Std::I8 : Std::ToString`

### impl `Std::I8 : Std::Zero`

### impl `Std::IO : Std::Functor`

### impl `Std::IO : Std::Monad`

### impl `Std::IO::IOFail : Std::Functor`

### impl `Std::IO::IOFail : Std::Monad`

### impl `[i1 : Std::Iterator, i2 : Std::Iterator] Std::Iterator::AppendIterator i1 i2 : Std::Iterator`

### impl `Std::Iterator::ArrayIterator a : Std::Iterator`

### impl `[i : Std::Iterator] Std::Iterator::ConsIterator i a : Std::Iterator`

### impl `Std::Iterator::CountUpIterator : Std::Iterator`

### impl `Std::Iterator::DynIterator : Std::Functor`

### impl `Std::Iterator::DynIterator : Std::Monad`

### impl `Std::Iterator::DynIterator a : Std::Add`

Concatenates two dynamic iterators.

### impl `[a : Std::Eq] Std::Iterator::DynIterator a : Std::Eq`

### impl `Std::Iterator::DynIterator a : Std::Iterator`

### impl `Std::Iterator::DynIterator a : Std::Zero`

Creates an empty dynamic iterator.

### impl `Std::Iterator::EmptyIterator a : Std::Iterator`

### impl `[i : Std::Iterator] Std::Iterator::FilterIterator i a : Std::Iterator`

### impl `[i : Std::Iterator] Std::Iterator::FilterMapIterator i a b : Std::Iterator`

### impl `[i2 : Std::Iterator, i1 : Std::Iterator] Std::Iterator::FlattenIterator i2 i1 : Std::Iterator`

### impl `[i : Std::Iterator] Std::Iterator::IntersperseIterator i a : Std::Iterator`

### impl `[i : Std::Iterator] Std::Iterator::MapIterator i a b : Std::Iterator`

### impl `[i1 : Std::Iterator, i2 : Std::Iterator] Std::Iterator::ProductIterator i1 i2 a b : Std::Iterator`

### impl `Std::Iterator::RangeIterator : Std::Iterator`

### impl `Std::Iterator::RangeStepIterator : Std::Iterator`

### impl `[i : Std::Iterator] Std::Iterator::ReverseIterator i a : Std::Iterator`

### impl `Std::Iterator::StateIterator s a : Std::Iterator`

### impl `[i : Std::Iterator] Std::Iterator::TakeIterator i : Std::Iterator`

### impl `[i : Std::Iterator] Std::Iterator::TakeWhileIterator i a : Std::Iterator`

### impl `[i1 : Std::Iterator, i2 : Std::Iterator] Std::Iterator::ZipIterator i1 i2 : Std::Iterator`

### impl `Std::Option : Std::Functor`

### impl `Std::Option : Std::Monad`

### impl `[a : Std::Eq] Std::Option a : Std::Eq`

### impl `[a : Std::ToString] Std::Option a : Std::ToString`

### impl `Std::Option::OptionIterator (Std::Option a) : Std::Iterator`

### impl `Std::Ptr : Std::Eq`

### impl `Std::Ptr : Std::ToString`

### impl `Std::Result e : Std::Functor`

### impl `Std::Result e : Std::Monad`

### impl `[e : Std::Eq, a : Std::Eq] Std::Result e a : Std::Eq`

### impl `[e : Std::ToString, a : Std::ToString] Std::Result e a : Std::ToString`

### impl `Std::String : Std::Add`

Concatenates two strings.

### impl `Std::String : Std::Eq`

### impl `Std::String : Std::LessThan`

### impl `Std::String : Std::LessThanOrEq`

### impl `Std::String : Std::ToString`

### impl `Std::String : Std::Zero`

The empty string.

### impl `Std::String::StringSplitIterator : Std::Iterator`

### impl `Std::Tuple2 t0 : Std::Functor`

### impl `Std::Tuple3 t0 t1 : Std::Functor`

### impl `Std::U16 : Std::Add`

### impl `Std::U16 : Std::Div`

### impl `Std::U16 : Std::Eq`

### impl `Std::U16 : Std::FromBytes`

### impl `Std::U16 : Std::FromString`

### impl `Std::U16 : Std::LessThan`

### impl `Std::U16 : Std::LessThanOrEq`

### impl `Std::U16 : Std::Mul`

### impl `Std::U16 : Std::Neg`

### impl `Std::U16 : Std::Rem`

### impl `Std::U16 : Std::Sub`

### impl `Std::U16 : Std::ToBytes`

### impl `Std::U16 : Std::ToString`

### impl `Std::U16 : Std::Zero`

### impl `Std::U32 : Std::Add`

### impl `Std::U32 : Std::Div`

### impl `Std::U32 : Std::Eq`

### impl `Std::U32 : Std::FromBytes`

### impl `Std::U32 : Std::FromString`

### impl `Std::U32 : Std::LessThan`

### impl `Std::U32 : Std::LessThanOrEq`

### impl `Std::U32 : Std::Mul`

### impl `Std::U32 : Std::Neg`

### impl `Std::U32 : Std::Rem`

### impl `Std::U32 : Std::Sub`

### impl `Std::U32 : Std::ToBytes`

### impl `Std::U32 : Std::ToString`

### impl `Std::U32 : Std::Zero`

### impl `Std::U64 : Std::Add`

### impl `Std::U64 : Std::Div`

### impl `Std::U64 : Std::Eq`

### impl `Std::U64 : Std::FromBytes`

### impl `Std::U64 : Std::FromString`

### impl `Std::U64 : Std::LessThan`

### impl `Std::U64 : Std::LessThanOrEq`

### impl `Std::U64 : Std::Mul`

### impl `Std::U64 : Std::Neg`

### impl `Std::U64 : Std::Rem`

### impl `Std::U64 : Std::Sub`

### impl `Std::U64 : Std::ToBytes`

### impl `Std::U64 : Std::ToString`

### impl `Std::U64 : Std::Zero`

### impl `Std::U8 : Std::Add`

### impl `Std::U8 : Std::Div`

### impl `Std::U8 : Std::Eq`

### impl `Std::U8 : Std::FromBytes`

### impl `Std::U8 : Std::FromString`

### impl `Std::U8 : Std::LessThan`

### impl `Std::U8 : Std::LessThanOrEq`

### impl `Std::U8 : Std::Mul`

### impl `Std::U8 : Std::Neg`

### impl `Std::U8 : Std::Rem`

### impl `Std::U8 : Std::Sub`

### impl `Std::U8 : Std::ToBytes`

### impl `Std::U8 : Std::ToString`

### impl `Std::U8 : Std::Zero`