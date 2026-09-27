Borrows a pointer to the data of a boxed value.

The returned pointer points to:

- if the value is a struct, the first field,
- if the value is an union, the data field (not the tag field).

The difference from `boxed_to_retained_ptr` is that this function returns a pointer to region where the payload of a boxed value is stored;
on the other hand, `boxed_to_retained_ptr` returns a pointer to the boxed value itself (which currently points to the reference counter of the boxed value).

The value is borrowed for the duration of the call, so the pointer is valid only while `borrower` runs.
It is not allowed to mutate a boxed value through the borrowed pointer. If you want to do so, use `mutate_boxed`.

See also: `borrow_boxed_io`, `mutate_boxed`, `mutate_boxed_io`.

# Parameters

* `borrower` - The function to call with the pointer.
* `value` - The boxed value to be borrowed.
