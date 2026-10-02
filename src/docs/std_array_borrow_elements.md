Calls a function with a pointer to the first element of the array's element buffer.

The array is borrowed for the duration of the call, so the pointer is valid only while `f` runs. The pointer must not be used to mutate the array; to do that, use `mutate_elements`.

# Parameters

* `f` - The function to call with the pointer to the first element.
* `x` - The array whose elements are borrowed.
