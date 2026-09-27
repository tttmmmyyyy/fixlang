Internal implementation of the `mutate_boxed` and `mutate_boxed_io` functions.

# Parameters

* `f` - The function that takes the pointer to the boxed value's data and returns the action to run.
* `x` - The boxed value to mutate.
* `ios` - The `IOState` to thread through the action.
