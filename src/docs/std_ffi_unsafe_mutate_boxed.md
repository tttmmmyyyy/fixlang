`x.mutate_boxed(io)` gets a pointer `ptr` to the data that `x` points to, executes `io(ptr)`, and then returns mutated `x` paired with the result of ``io(ptr)``.

The IO action `io(ptr)` is expected to modify the value of `x` through the obtained pointer. 
Do not perform any IO operations other than mutating the value of `x`.

This function first clones the value if `x` is not unique.

At the moment, it is not specified what pointer is obtained for a union, so do not use this function with unions.

This function is unsafe in the sense that it returns different `Ptr` values created by the same expression.