`v.count_leading_zeros` counts the zero bits of `v` above its most significant one bit: `1_U8.count_leading_zeros` is `7_U8`.

The sign bit of a signed type counts as the most significant bit, so a negative `v` gives zero. A `v` of zero gives the number of bits in its type.

# Parameters

* `v` - The value whose bits are counted.
