#$ test: compile-fail
#$ rules: LEX-24
# D-312 — `[LEX-24]`: "If that type is unsigned the program is rejected with
# `E2010`". `x: u8 = -1` compiled, and held 255. A negative literal past a
# signed type's least value is `E2010` too, naming the whole literal.

fn main():
    x: u8 = -1    #$ error[E2010]: the literal `-1` does not fit in `u8`
    y: u32 = -5    #$ error[E2010]: the literal `-5` does not fit in `u32`
    z: i8 = -129    #$ error[E2010]: the literal `-129` does not fit in `i8`
    println(x, y, z)
