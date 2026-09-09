#$ test: run-pass
#$ rules: RNG-4, RNG-10
# `[RNG-4]` tracks "a known range for every numeric expression it can …
# arithmetic on operands with known ranges", and `[RNG-10]`(d) makes "a value
# whose `[RNG-4]` range is contained in the target's" one of the five ways a
# range value arises in Safe code — with no check, because every value the
# expression can take is already a value of the target.
#
# The fact doing the work here is not the arithmetic: it is that `r` is a
# `Roughness` and therefore in `0.0 ..= 1.0`, which `[RNG-9]` makes an
# invariant and `[RNG-4]` may assume. So `r * 0.5` is in `0.0 ..= 0.5`, which
# `Roughness` contains.
#
# Until this existed, (d) admitted a constant and nothing else, and this
# program was `E2215` (D-018).

type Roughness = f32 in 0.0 ..= 1.0

fn main():
    r: Roughness = 0.5
    half = r * 0.5
    back: Roughness = half
    v: f32 = back
    println(v)
#$ stdout: 0.25
