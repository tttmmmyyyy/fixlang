Traverses all values reachable from the given value, and changes the reference counters of them into multi-threaded mode.

Building a program that calls this function requires multi-threading to be enabled by the `--threaded` compiler option or by the `threaded` field of the project file of the program being built. A library that calls this function is used by turning multi-threading on there.

# How a value reaches another thread

A value reaches another thread as a pointer: `Std::FFI::boxed_to_retained_ptr` turns a boxed value into a `Ptr`, and that pointer is what a threading library such as pthread is handed. A value of an unboxed type is wrapped in `Std::Box` first, so that there is a boxed value to take the pointer of. The whole sequence is therefore: wrap the value if its type is unboxed, call this function on it, call `Std::FFI::boxed_to_retained_ptr` on what this function returns, and hand the pointer over.

# Hand the result over immediately

**Pass the value this function returns straight to `Std::FFI::boxed_to_retained_ptr`, and do nothing else with it in between.** The multi-threaded mode is a state stored in the object, and an operation on the value can put the object back into single-threaded mode: an operation that finds the object uniquely referenced updates it in place, which is sound only for an object no other thread reaches, and it records that by returning the object to single-threaded mode. Reading the value counts as such an operation, since asking whether a value is uniquely referenced is itself an operation on the object.

The mode this sets is what every thread's reference counting reads, so a value already reachable from a second thread when the call runs is counted in one mode by one thread and in another by the other, and the counts each thread makes are lost to the other.

# Sending on a value received from another thread

A value that arrived from another thread is in multi-threaded mode when it arrives, and any operation this thread performs on it can return it to single-threaded mode by the rule above. **Call this function again on a value being sent on, however it was obtained.** The rule is the same one as above: what is handed to `Std::FFI::boxed_to_retained_ptr` is what this function has just returned.

# Parameters

* `value` - The value to make multi-threaded.
