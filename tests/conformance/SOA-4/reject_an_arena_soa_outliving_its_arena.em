#$ test: compile-fail
#$ rules: SOA-4, ARN-5
# `[SOA-4]` — an `ArenaSoA` takes its storage from the arena and borrows it for its life
# (`[ARN-1]`): it cannot be returned past the arena, and the arena cannot be reset while it lives.
from std.collections import ArenaSoA

struct Point:
    x: f32

fn escape() -> ArenaSoA[Point]:
    arena = Arena.with_capacity(64)
    return ArenaSoA[Point].with_capacity(arena, 1)  #$ error[E3061]: arena allocation cannot outlive `arena`

fn main():
    arena = Arena.with_capacity(64)
    points = ArenaSoA[Point].with_capacity(arena, 1)
    arena.reset()  #$ error[E3021]: `arena` is borrowed here and mutably borrowed elsewhere
    println(points.len())
