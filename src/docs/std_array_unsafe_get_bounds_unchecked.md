Gets the element of an array at the specified index, omitting the bounds check.

The caller must ensure `idx` is in range `[0, size)`; an out-of-range index causes undefined behavior. Use it in a loop that reads elements at indices known to be in range, when the compiler cannot prove that they are.

# Parameters

* `idx` - The index of the element to get.
* `array` - The array to get the element from.