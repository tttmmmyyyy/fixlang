Returns a pointer to the function of type `void (*)(void*)` which retains a boxed value of type `a`.
This function is used to retain a pointer obtained by `_unsafe_get_retained_ptr_of_boxed_value`.

For the reason that this function requires a value of type `Lazy a`, not of `a`, see the document for `unsafe_get_release_function_of_boxed_value`.