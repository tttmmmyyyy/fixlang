Checks whether the array's storage is uniquely referenced, and returns the result paired with the array itself.

This reads the reference count of the array's element storage in place, without retaining it. It is the array counterpart of `Std::unsafe_is_unique`, which does not apply to `Array` because an `Array` value is unboxed.

NOTE: Changing outputs of your function depending on uniqueness breaks the referential transparency of the function.

NOTE: This function's return value may change depending on the optimization level, because optimizations may change a value from being shared to being unique.

# Parameters

* `array` - The array to check for storage uniqueness.
