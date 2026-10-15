#$ test: run-pass
#$ rules: CT-4, DET-4, TYP-8
#$ stdout: true true true 1267650600228229401496703205376 -128
# `[CT-4]` — compile time computes as the target does: `std.math.det`'s `sin` gives the bits it
# gives at run time, an `f32` rounds as an `f32`, a 128-bit integer is exact, and the overflow
# policy of a function is its own at compile time too.

import math.det

fn third() -> f32:
    return (1.0 as f32) / 3.0

fn big() -> i128:
    return (1 as i128) << 100

@overflow(wrap)
fn wrapped() -> i8:
    x: i8 = 127
    return x + 1

const DS: f64 = det.sin(1.0)

fn main():
    println(DS == det.sin(1.0), comptime(third()) == third(), comptime(big()) == big(), comptime(big()), comptime(wrapped()))
