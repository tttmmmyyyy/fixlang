Checks whether `0 <= idx < size` holds when `idx` and `size` are given.

If the condition holds, returns the value of `idx` as is.

If the condition does not hold, aborts the program.

This function is a built-in function provided to centralize the display of error messages related to array bounds checking.
Additionally, in the future, if a compiler option that disables array bounds checking is added, it will be realized by replacing the implementation of this function.

# Parameters
* `idx` - The index to check.
* `size` - The size to check the index against.