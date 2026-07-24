Grows the length of an array over uninitialized slots, without validating the given length.

This function clones the given array if it is shared. The caller must ensure `new_len >= size` and `new_len <= capacity`, and must fill the new slots before they are read; the element type must contain no boxed value. Violating any of these causes undefined behavior.

# Parameters

* `new_len` - The new length.
* `array` - The array to grow.
