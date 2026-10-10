The bits of `v` in the IEEE 754 binary format, as an unsigned integer of the same width: a `U64` for an `F64`, and a `U32` for an `F32`. `1.0.to_bits` is `0x3FF0000000000000_U64`, and `-0.0.to_bits` is `0x8000000000000000_U64`.

`from_bits` reads the bits back: `v.to_bits.from_bits` has the bits of `v`.

# Parameters

* `v` - The number whose bits are read.
