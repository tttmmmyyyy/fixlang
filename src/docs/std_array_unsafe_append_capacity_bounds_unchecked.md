Appends the range `src[begin, end)` to the end of `dst`, omitting the capacity check.

`dst` is cloned first if it is shared. When `src` is uniquely owned and the whole of it is being appended, the elements are moved without any reference counting; otherwise each is retained as it is copied. The caller must ensure `0 <= begin <= end <= src.size` and `dst.size + (end - begin) <= dst.capacity`; violating either causes undefined behavior.

# Parameters

* `src` - The array to append from.
* `begin` - The start index of the range in `src`.
* `end` - The end index of the range in `src`.
* `dst` - The array to append to.
