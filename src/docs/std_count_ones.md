`v.count_ones` counts the one bits of `v`: `11_U8.count_ones` is `3_U8`.

The sign bit of a signed type counts as one of its bits, so `-1_I8.count_ones` is `8_I8`.

# Parameters

* `v` - The value whose bits are counted.
