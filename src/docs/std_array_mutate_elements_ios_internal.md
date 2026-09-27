Internal implementation of the `mutate_elements` and `mutate_elements_io` functions.

# Parameters

* `f` - The function that takes the pointer to the first element and returns the action to run.
* `x` - The array to mutate.
* `ios` - The `IOState` to thread through the action.
