Gets a pointer to the data of a boxed value.

NOTE: 
This function is unsafe in that if the call `v._get_boxed_ptr` is the last usage of `v`, then this function deallocates `v` and returns a dangling pointer.
To avoid this issue, use `borrow_boxed`, `borrow_boxed_io`, `mutate_boxed`, or `mutate_boxed_io` instead.

# Parameters

* `value` - The boxed value to get the pointer to.