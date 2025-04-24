Updates an array by applying a function to the element at the specified index.

This function clones the given array if it is shared.

If you call `arr.mod(i, f)` when both of `arr` and `arr.@(i)` are unique, it is assured that `f` receives the element value which is unique. 

# Parameters

* `i` - The index of the element to modify.
* `modifier` - The function to apply to the element.
* `array` - The array to modify.