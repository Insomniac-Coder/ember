#$ test: compile-fail
#$ rules: TYP-6
# `[TYP-6]` — only `u8` converts to `char` with `as`, and a `char` converts
# only to `u32`; the rest go through `char.from_u32`.

fn main():
    n: u32 = 65
    c = 'a'
    println(n as char)  #$ error[E2020]: `u32` cannot be cast to `char` with `as`
    println(c as i64)  #$ error[E2020]: `char` cannot be cast to `i64` with `as`
