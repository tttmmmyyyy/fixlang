Traverses all values reachable from the given value, and changes the reference counters of them into multi-threaded mode.

Building a program that calls this function requires multi-threading to be enabled by the `--threaded` compiler option or by the `threaded` field of the project file of the program being built. A library that calls this function is used by turning multi-threading on there.

Call this before the value becomes reachable from another thread, and let the call finish before the pointer is handed over. The mode this sets is what every thread's reference counting reads, so a value already reachable from a second thread when the call runs is counted in one mode by one thread and in another by the other, and the counts each thread makes are lost to the other.

# Parameters

* `value` - The value to make multi-threaded.