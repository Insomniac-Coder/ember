#$ test: compile-fail
#$ rules: LAY-2, SIMD-1
#$ profiles: debug
# `[LAY-2]`, `[SIMD-1]` — a SIMD vector is aligned to its size, an alignment its type states as
# `@align` states one; a `@packed` type leaves no padding, so it holds no vector: `E0104`, as for an
# `@align` type (ODR-138, ODR-157).
from std.simd import f32x4

@packed
struct Particle:
    tag: u8
    velocity: f32x4  #$ error[E0104]: field `velocity` of a `@packed` type holds a SIMD vector

fn main():
    println(size_of[Particle]())
