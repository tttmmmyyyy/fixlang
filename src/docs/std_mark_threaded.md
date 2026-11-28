Traverses all values reachable from the given value, and changes the reference counters of them into multi-threaded mode.

This function is deprecated. Use `Std::FFI::pack_for_thread` instead: it changes the mode and returns the pointer to hand over in one call, so no operation can come between the two.

Building a program that calls this function requires multi-threading to be enabled by the `--threaded` compiler option or by the `threaded` field of the project file of the program being built. A library that calls this function is used by turning multi-threading on there.

# Using this function

Code that still uses this function hands a value over by calling `Std::FFI::boxed_to_retained_ptr` on what this function returns, wrapping a value of an unboxed type in `Std::Box` first. **Pass the value this function returns straight to `Std::FFI::boxed_to_retained_ptr`, and do nothing else with it in between**: an operation on the value, reading it included, can put the object back into single-threaded mode. A value being sent on to a further thread goes through this function again, however it was obtained.

# Parameters

* `value` - The value to make multi-threaded.
