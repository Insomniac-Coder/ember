#$ test: run-fail
#$ rules: STD-21, PAN-1
#$ panics: normalize of a zero vector; use normalize_or_zero
#$ stdout: Vec2(x=0.0, y=0.0)
#$ profiles: debug, release, shipping
# `[STD-21]` — a zero vector has no direction, so `normalize` panics;
# `normalize_or_zero` gives it back.

from std.math import Vec2

fn main():
    v = Vec2(3, 4) - Vec2(3, 4)
    println(v.normalize_or_zero())
    println(v.normalize())
