#$ test: run-pass
#$ rules: STD-1, EFF-5
#$ profiles: debug, development
#$ stdout: 3.0 4.0
#$ 5.0
#$ 2.0
#$ 10
# `[STD-1]` — every function in `std.core`, `std.mem`, `std.math`, `std.simd` and `std.arena` is
# `@noalloc` unless its documentation says it allocates: the compiler holds each to that as to a
# written `@noalloc` (`E4001`), so a `@noalloc` function may call them. Here vector and matrix
# maths (including `transpose` and `inverse`, whose index panics are plain text, `[PAN-2]`), a
# vector type and the standard library's numbers, from inside `@noalloc`.
import std.arena
import std.mem
from std.math import Vec2, Mat2
from std.simd import i32x4

@noalloc
fn work(v: Vec2, m: Mat2) -> f32:
    w = m.transpose().inverse() * v
    return w.length() + m.col(1).y

@noalloc
fn lanes(a: i32x4) -> int:
    return (a + a).reduce_add() as int

fn main():
    v = Vec2(3.0, 4.0)
    println(v.x, v.y)
    println(v.length())
    m = Mat2(Vec2(1.0, 0.0), Vec2(0.0, 1.0))
    println(work(Vec2(0.6, 0.8), m))
    println(lanes(i32x4.from_array([1, 1, 1, 2])))
