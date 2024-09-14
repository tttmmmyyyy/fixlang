This function checks if a value is uniquely referenced by a name, and returns the result paired with the given value itself. An unboxed value is always considered unique.

NOTE: Changing outputs of your function depending on uniqueness breaks the referential transparency of the function. If you want to abort when a value is shared, consider using `Debug::assert_unique` instead.

Example: 

```
module Main;

import Debug;

main : IO ();
main = (
    // For unboxed value, it returns true even if the value is used later.
    let int_val = 42;
    let (unique, _) = int_val.unsafe_is_unique;
    let use = int_val + 1;
    eval assert_eq(|_|"fail: int_val is shared", unique, true);

    // For boxed value, it returns true if the value isn't used later.
    let arr = Array::fill(10, 10);
    let (unique, arr) = arr.unsafe_is_unique;
    let use = arr.@(0); // This `arr` is not the one passed to `is_unique`, but the one returned by `is_unique`.
    eval assert_eq(|_|"fail: arr is shared", unique, true);

    // Fox boxed value, it returns false if the value will be used later.
    let arr = Array::fill(10, 10);
    let (unique, _) = arr.unsafe_is_unique;
    let use = arr.@(0);
    eval assert_eq(|_|"fail: arr is unique", unique, false);

    pure()
);
```