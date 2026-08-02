Traverses all values reachable from the given value, and changes the reference counters of them into multi-threaded mode.

Building a program that calls this function requires multi-threading to be enabled by the `--threaded` compiler option or by the `threaded` field of the project file of the program being built. A library that calls this function is used by turning multi-threading on there.

# Parameters

* `value` - The value to make multi-threaded.