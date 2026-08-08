Copies the range `src[begin, end)` to the end of `dst`, omitting the capacity check and the bounds check.

`dst` is cloned first if it is shared. `src` is borrowed: each element it holds is retained as it is copied, and `src` itself is left to its caller. The caller must ensure `0 <= begin <= end <= src.size` and `dst.size + (end - begin) <= dst.capacity`; violating either causes undefined behavior.

# Parameters

* `src` - The array to copy from.
* `begin` - The start index of the range in `src`.
* `end` - The end index of the range in `src`.
* `dst` - The array to copy to.
