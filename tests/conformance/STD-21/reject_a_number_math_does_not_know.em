#$ test: compile-fail
#$ rules: STD-21, TYP-17
# ODR-038 — `std.math`'s functions take what `Number` lists: every integer
# and decimal type. Text is not a number, and an integer's answer is an
# `f64`, which does not narrow to `f32` by itself.

import math

fn main():
    println(math.sqrt("hi"))    #$ error[E2040]: `str` does not implement `std.math.Number`, which `T` requires
    n: i32 = 9
    narrow: f32 = math.sqrt(n)    #$ error[E2020]: expected `f32`, found `f64`
    println(narrow)
