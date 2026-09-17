`v.shift_right(bits)` shifts `v` to right by `bits` bits.

The caller must ensure that `bits` is at least zero and less than the number of bits of the type of `v`. Outside that range the result is unspecified, and `--check-shift-amount` stops the program where the amount is outside it.

# Parameters

* `bits` - The number of bits to shift.
* `v` - The value to shift.