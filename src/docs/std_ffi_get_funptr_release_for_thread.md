Returns a pointer to the function of type `void (*)(void*)` which releases a pointer that `pack_for_thread` returned for a value of type `a`.

The type `a` can be any type, boxed or unboxed: the function releases the object the pointer points to, which is a `Std::Box` for an unboxed type.

For the reason that this function requires a value of type `Lazy a`, not of `a`, see the document for `get_funptr_release`.

# Parameters

* `lazy_value` - The lazy value to indicate the type of the value the pointer was made from.