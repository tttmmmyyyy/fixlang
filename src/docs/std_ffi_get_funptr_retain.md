Returns a pointer to the function of type `void (*)(void*)` which retains a boxed value of type `a`.
This function is used to retain a pointer obtained by `boxed_to_retained_ptr`.

For the reason that this function requires a value of type `Lazy a`, not of `a`, see the document for `get_funptr_release`.