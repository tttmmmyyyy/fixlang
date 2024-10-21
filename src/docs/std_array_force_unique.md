Force the uniqueness of an array.
If the given array is shared, this function returns the cloned array.

@deprecated
This function is deprecated because it is fragile when the "common expression elimination" optimization is implemented in the future. 
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