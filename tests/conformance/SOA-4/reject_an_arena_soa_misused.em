#$ test: compile-fail
#$ rules: SOA-4, ARN-5
# `[SOA-4]` — under `[ARN-5]`'s rules an `ArenaSoA`'s elements must not need drop, and it never
# grows: it has no `reserve`.
from std.collections import ArenaSoA

struct Named:
    name: String

struct Point:
    x: f32

fn main():
    arena = Arena.with_capacity(1024)
    named = ArenaSoA[Named].with_capacity(arena, 2)  #$ error[E3090]: `ArenaSoA[Named]` requires `!needs_drop(Named)`
    points = ArenaSoA[Point].with_capacity(arena, 2)
    points.reserve(4)  #$ error[E2072]: has no method named `reserve`
