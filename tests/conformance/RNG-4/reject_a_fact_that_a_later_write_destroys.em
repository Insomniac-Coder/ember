#$ test: compile-fail
#$ rules: RNG-4, RNG-10
# A range fact belongs to a value, not to a name. `half` is written again
# before it is used, so what `[RNG-4]` derived at the initialiser says nothing
# about what is there now, and the fact is dropped rather than joined.
#
# The conservative direction is a check that gets emitted; the other direction
# would be a range value that never passed one, which `[RNG-9]` makes
# undefined behaviour rather than merely wrong.

type Roughness = f32 in 0.0 ..= 1.0

fn wild() -> f32:
    return 9.0

fn main():
    r: Roughness = 0.5
    half = r * 0.5
    half = wild()
    back: Roughness = half    #$ error[E2215]: a `Roughness` cannot be built from a value this is not known to be in range
    v: f32 = back
    println(v)
