Force the uniqueness of an array.
If the given array is shared, this function returns the cloned array.

This function is unsafe and is intended for internal use within the standard library only. General users should not call it directly.

It is fragile when the "common subexpression elimination" (CSE) optimization is implemented in the future. Consider the following example:

```
f : Array a -> Array a;
f = |arr| arr._unsafe_force_unique.do_something_for_unique_array;

let x = [1, 2, 3];
let y = f(x);
let z = f(x);
```

When this function `f` is inlined, the code will be as follows.

```
let x = [1, 2, 3];
let y = x._unsafe_force_unique.do_something_for_unique_array;
let z = x._unsafe_force_unique.do_something_for_unique_array;
```

Here, if CSE is applied to the two `x._unsafe_force_unique`, the code will call `do_something_for_unique_array` with a non-unique array.

```
let x = [1, 2, 3];
let x = x._unsafe_force_unique;
let y = x.do_something_for_unique_array; // Here `x` is not unique
let z = x.do_something_for_unique_array;
```

To use this function safely, the inlining of `f` above must be suppressed. Since it is uncertain whether a function attribute such as "noinline" will be added in the future, this function is reserved for carefully audited internal use.
