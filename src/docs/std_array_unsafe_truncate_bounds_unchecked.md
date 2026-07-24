Truncates an array to `new_len` elements, releasing the dropped tail, omitting the size check.

This function clones the given array if it is shared. The caller must ensure `0 <= new_len <= size`; a `new_len` outside that range causes undefined behavior.

# Parameters

* `new_len` - The number of leading elements to keep.
* `array` - The array to truncate.
