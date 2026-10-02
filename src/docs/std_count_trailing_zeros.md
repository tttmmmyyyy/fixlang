`v.count_trailing_zeros` counts the zero bits of `v` below its least significant one bit: `12_U8.count_trailing_zeros` is `2_U8`.

If `v` is zero, the count is the number of bits in its type: `0_U8.count_trailing_zeros` is `8_U8`.

# Parameters

* `v` - The value whose bits are counted.
