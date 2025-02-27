`x.mutate_boxed(io)` gets a pointer `ptr` to the data that `x` points to, executes `io(ptr)`, and then returns mutated `x` paired with the result of ``io(ptr)``.

The IO action `io(ptr)` is expected to modify the value of `x` through the obtained pointer. 
Do not perform any IO operations other than mutating the value of `x`.

For more details on the pointer passed to `io`, see the document of `borrow_boxed`.

This function first clones the value if `x` is not unique.

See also: `borrow_boxed`, `mutate_boxed_io`, `mutate_boxed`.