Swaps the two elements of an array at indices `i` and `j`, omitting the bounds check.

This function clones the given array if it is shared. The caller must ensure `i` and `j` are in range `[0, size)`; an out-of-range index causes undefined behavior.

# Parameters

* `i` - The index of the first element.
* `j` - The index of the second element.
* `array` - The array to modify.
