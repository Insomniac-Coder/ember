#$ test: compile-fail
#$ rules: CT-7, CT-1, TYP-8, TYP-10
# `[CT-1]`, `[CT-7]` — a `const` is evaluated while compiling, so what would
# panic at run time is `E6004` at the declaration, with the panic's message:
# an overflow names its operator (`[TYP-8]`), a shift amount outside
# `0 ≤ n < width` panics (`[TYP-10]`), and so does an unsigned negation of
# anything but 0 (D-314). A use of a refused constant says nothing more.

const BIG: i32 = 2147483647 + 1    #$ error[E6004]: integer overflow in `+`
const ZERO_DIV: i64 = 7 // (3 - 3)    #$ error[E6004]: division by zero
const ZERO_REM: u16 = 7 % (2 - 2)    #$ error[E6004]: division by zero
const TOO_FAR: u32 = 1 << 32    #$ error[E6004]: integer overflow in `shift`
const NEGATIVE: u8 = -(3 - 2)    #$ error[E6004]: integer overflow in `-`
const LEAST: i64 = i64.MIN // -1    #$ error[E6004]: integer overflow in `//`
const PRODUCT: i8 = 16 * 8    #$ error[E6004]: integer overflow in `*`

fn main():
    println(BIG, ZERO_DIV, ZERO_REM, TOO_FAR, NEGATIVE, LEAST, PRODUCT)
