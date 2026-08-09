Appends the whole of `src` to the end of `dst`, omitting the capacity check.

`dst` is cloned first if it is shared. When `src` is uniquely owned, its elements are moved without any reference counting; otherwise each is retained as it is copied. The caller must ensure `dst.size + src.size <= dst.capacity`; violating it causes undefined behavior.

# Parameters

* `src` - The array to append.
* `dst` - The array to append to.
