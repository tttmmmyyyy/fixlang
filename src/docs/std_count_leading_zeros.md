`v.count_leading_zeros` counts the zero bits of `v` above its most significant one bit: `1_U8.count_leading_zeros` is `7_U8`.

The sign bit of a signed type counts as the most significant bit, so the count is zero for a negative `v`. If `v` is zero, the count is the number of bits in its type: `0_U8.count_leading_zeros` is `8_U8`.

# Parameters

* `v` - The value whose bits are counted.
