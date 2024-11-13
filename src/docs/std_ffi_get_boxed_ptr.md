Returns a pointer to the data of a boxed value.

The returned pointer points to:
- if the value is an `Array`, the first element of the array,
- if the value is a struct, the first field,
- if the value is an union, the data field (not the tag field).

The difference from `boxed_to_retained_ptr` is that this function returns a pointer to region where the payload of a boxed value is stored;
on the other hand, `boxed_to_retained_ptr` returns a pointer to the boxed value itself (which currently points to the reference counter of the boxed value).

NOTE: 
This function is unsafe in that if the call `v._get_boxed_ptr` is the last usage of `v`, then this function deallocates `v` and returns a dangling pointer.
To avoid this issue, use `borrow_boxed`, `borrow_boxed_io`, `mutate_boxed`, or `mutate_boxed_io` instead.