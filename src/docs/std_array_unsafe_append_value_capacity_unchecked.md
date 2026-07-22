Appends `count` copies of `value` to the end of an array, omitting the capacity check.

This function clones the given array if it is shared. The caller must ensure `count >= 0` and `size + count <= capacity`; violating either causes undefined behavior.

# Parameters

* `value` - The value to append.
* `count` - The number of copies to append.
* `array` - The array to append to.
