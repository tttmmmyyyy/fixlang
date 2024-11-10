Returns a retained pointer to a boxed value.
This function is used to share ownership of Fix's boxed values with foreign languages.

To get back the boxed value from the retained pointer, use `from_retained_ptr`.
To release / retain the value in a foreign language, call the function pointer obtained by `get_funptr_release` or `get_funptr_retain` on the pointer.

Note that the returned pointer points to the control block allocated by Fix, and does not necessary points to the data of the boxed value.
If you want to get a pointer to the data of the boxed value, use `borrow_boxed`.