#$ test: run-pass
#$ rules: MOD-5
#$ stdout: 3
#$ assert-c: !contains("em_2i88floordiv(")
#$ assert-c: !contains("em_4u1283rem(")
#$ assert-c: !contains("em_4i1287default(")
#$ assert-c: contains("em_3i328floordiv(")
# `[MOD-5]` — the standard library is available to every program, and only
# what a program reaches is emitted. That holds for `std`'s extensions of the
# built-in types too, whose bodies are named after the type (`i8.floordiv`,
# from `std.core`'s `extend i8 implements FloorDiv[NonZero[i8]]`): D-339 emitted
# each of them into every program. This one divides one `i32` by a `NonZero`.

from std.core import NonZero

fn main():
    x: i32 = 7
    println(x // NonZero[i32].new(2).unwrap())
