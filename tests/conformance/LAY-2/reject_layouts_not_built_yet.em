#$ test: compile-fail
#$ rules: LAY-2, ATT-6
#$ profiles: debug
# `[LAY-2]`, `[ATT-6]` — two layouts the rule defines are not built, and are refused with `E0900`
# rather than laid out some other way: a SIMD vector (`[SIMD-1]`, aligned to its size) in a
# `@packed` type, which MSVC will not pack, and `@align(N)` on an enum without payloads, which C
# holds as its bare discriminant integer.
from std.simd import f32x4

@packed
struct Particle:
    tag: u8
    velocity: f32x4  #$ error[E0900]: field `velocity` of a `@packed` type holds a SIMD vector

@align(64)
enum Mode:  #$ error[E0900]: `@align` on an enum without payloads is not implemented yet
    Slow
    Fast

fn main():
    println(size_of[Particle]())
