#$ test: compile-fail
#$ rules: LAY-2
#$ profiles: debug
# `[LAY-2]` — `@repr(T)` makes `T` the discriminant's type, so a discriminant, written or counted
# on from the one before, that `T` cannot hold is `E2010`: it is never truncated.

@repr(u8)
enum Byte:
    Low = 254
    High
    Over  #$ error[E2010]: discriminant 256 of `Over` does not fit `u8`

@repr(i8)
enum Signed:
    Under = -129  #$ error[E2010]: discriminant -129 of `Under` does not fit `i8`

@repr(u16)
enum Unsigned:
    Negative = -1  #$ error[E2010]: discriminant -1 of `Negative` does not fit `u16`

fn main():
    println(size_of[Byte]())
