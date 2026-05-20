Gets the capacity of an array.

The capacity of an array is the number of elements that can be stored in the currently allocated memory region.
Up to this number of elements, you can add elements without additional memory allocation.
If you try to add more elements than this number, the array will automatically reallocate memory,
which may be a costly operation.

# Parameters

* `array` - The array to get the capacity of.
