This function checks if a boxed value is uniquely referenced by a name, and returns the result paired with the given value itself.

For arrays, use `Array::_unsafe_is_storage_unique`, which checks the array's storage.

Example: 
```
module Main;

type Resource = box struct { id : I64 };

main : IO ();
main = (
    // For a boxed value, it returns true if the value isn't used later.
    let res = Resource { id : 42 };
    let (unique, res) = res.unsafe_is_unique;
    let use = res.@id; // This `res` is the one returned by `unsafe_is_unique`, not the one passed to it.
    eval use; // `eval` ensures that the computation of `use` is not optimized away
    assert_eq(|_|"fail: res is shared", unique, true);;

    // For a boxed value, it returns false if the value will be used later.
    let res = Resource { id : 42 };
    let (unique, _) = res.unsafe_is_unique;
    let use = res.@id;
    eval use; // `eval` ensures that the computation of `use` is not optimized away
    assert_eq(|_|"fail: res is unique", unique, false);;

    pure()
);
```

NOTE: Changing outputs of your function depending on uniqueness breaks the referential transparency of the function. If you want to assert that a value is unique, consider using `Debug::assert_unique` instead.

NOTE: This function's return value may change depending on the optimization level. This is because optimizations may eliminate unnecessary computations and change a value from being shared to being unique.

# Parameters

* `value` - The value to check for uniqueness.
