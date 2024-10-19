Returns a pointer to the data of a boxed value.

The returned pointer points to the first element of the array if the value is an `Array`, and to the first field if the value is a struct.
At the moment, it is not specified what pointer is returned for a union, so do not use this function with unions.

The difference from `unsafe_get_retained_ptr_of_boxed_value` is that this function returns a pointer to region where the payload of a boxed value is stored;
on the other hand, `unsafe_get_retained_ptr_of_boxed_value` returns a pointer to the boxed value itself (i.e., the control block of the value).

Note that if the call `v._unsafe_get_boxed_data_ptr` is the last usage of `v`, then this function deallocates `v` and returns a dangling pointer.
To avoid issues caused by this, use `unsafe_borrow_boxed_data_ptr` instead.

This function is unsafe in the sense that it returns different `Ptr` values created by the same expression.