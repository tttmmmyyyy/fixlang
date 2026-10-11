Sets an element of an array at the specified index, omitting the bounds check.

This function clones the given array if it is shared, and releases the element previously at the index. The caller must ensure `idx` is in range `[0, size)`; an out-of-range index causes undefined behavior. Alternatively, write the program with `set`, test it with the bounds checks on, and build it with `--no-runtime-check` where it needs the speed; that option removes the bounds checks of every array access.

# Parameters

* `idx` - The index of the element to set.
* `value` - The value to set.
* `array` - The array to modify.
