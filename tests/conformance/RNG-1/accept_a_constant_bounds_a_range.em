#$ test: run-pass
#$ rules: RNG-1, CT-1
#$ profiles: debug, release, shipping
#$ stdout: true false true false
#$ true false
# `[RNG-1]` — an endpoint is a constant of the representation type: a
# literal, or a `const`, however it is written (`-20`, `LIMIT`, `-LIMIT`)
# and wherever it is declared. D-329: a `const` endpoint was always `E2212`,
# since range types are collected before constants are.

type Offset = i32 in -LIMIT ..= LIMIT
type Level = f32 in FLOOR ..= 1.0

const LIMIT = 20
const FLOOR: f32 = -0.5

fn main():
    println(Offset.checked(-20).is_ok(), Offset.checked(21).is_ok(), Offset.checked(20).is_ok(), Offset.checked(-21).is_ok())
    println(Level.checked(-0.5).is_ok(), Level.checked(-0.75).is_ok())
