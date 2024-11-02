Returns a retained pointer to a boxed value.
This function is used to share ownership of Fix's boxed values with foreign languages.

To get back the boxed value from the retained pointer, use `unsafe_get_boxed_value_from_retained_ptr`.
To release / retain the value in a foreign language, call the function pointer obtained by `unsafe_get_release_function_of_boxed_value` or `unsafe_get_retain_function_of_boxed_value` on the pointer.

Note that the returned pointer points to the control block allocated by Fix, and does not necessary points to the data of the boxed value.
If you want to get a pointer to the data of the boxed value, use `unsafe_borrow_boxed_ptr`.