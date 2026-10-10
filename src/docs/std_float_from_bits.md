The number whose bits in the IEEE 754 binary format are `bits`: an `F64` from a `U64`, and an `F32` from a `U32`. `F64::from_bits(0x3FF0000000000000_U64)` is `1.0`.

Every value of `bits` is the bits of a number: a finite number, an infinity or a NaN. `bits.from_bits.to_bits` is `bits`, including the sign and the other bits of a NaN. An arithmetic operation on a NaN may change those bits, so they are kept only while the NaN passes through no operation.

# Parameters

* `bits` - The bits of the number.
