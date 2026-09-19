`v.shift_right(bits)` shifts `v` to the right by `bits` bits.

`bits` must be at least zero and less than the number of bits in the type of `v`. Outside that range the result is an unspecified value of that type, and `--check-integer-operations` stops the program.

# Parameters

* `bits` - The number of bits to shift.
* `v` - The value to shift.