Borrows a pointer to the data of a boxed value.

The returned pointer points to:

- if the value is a struct, the first field,
- if the value is a union, the data field (not the tag field).

Unlike `boxed_to_retained_ptr`, which returns a pointer to the boxed value itself (currently its reference counter), this function returns a pointer to the region where the payload of the boxed value is stored.

The value is borrowed for the duration of the call, so the pointer is valid only while `f` runs.
It is not allowed to mutate a boxed value through the borrowed pointer. If you want to do so, use `mutate_boxed`.

See also: `borrow_boxed_io`, `mutate_boxed`, `mutate_boxed_io`.

# Parameters

* `f` - The function to call with the pointer.
* `x` - The boxed value to be borrowed.
