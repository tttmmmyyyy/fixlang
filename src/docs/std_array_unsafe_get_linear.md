Gets a value from an array at the specified index.

This function differs from `unsafe_get` in that the return value is a pair of the array and the value obtained.
This can reduce the number of reference count increments and decrements for the array.

This function is unsafe in the following sense:
* Does not check if the array index is within bounds.
* The returned value is not retained.