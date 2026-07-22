Sets an array's capacity to `new_cap`, omitting the check that the new capacity fits the current elements.

A uniquely owned array is resized in place with `realloc` without touching its elements; a shared array is copied into a new allocation. The caller must ensure `new_cap >= size`; a smaller capacity causes undefined behavior.

# Parameters

* `new_cap` - The new capacity.
* `array` - The array to resize.
