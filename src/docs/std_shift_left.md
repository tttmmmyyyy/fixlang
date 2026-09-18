`v.shift_left(bits)` shifts `v` to left by `bits` bits.

A shift is defined for `bits` at least zero and less than the number of bits of the type of `v`. Outside that range the result is an unspecified value of that type, and `--check-integer-operations` stops the program there.

# Parameters

* `bits` - The number of bits to shift.
* `v` - The value to shift.