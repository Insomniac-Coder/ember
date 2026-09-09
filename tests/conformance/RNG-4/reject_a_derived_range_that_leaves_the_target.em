#$ test: compile-fail
#$ rules: RNG-4, RNG-10
# The other side. `r * 4.0` is in `0.0 ..= 4.0`, which `Roughness` does not
# contain, so `[RNG-10]`(d) does not apply and the closed construction set
# leaves `checked` and `clamped` — which is what the help names.

type Roughness = f32 in 0.0 ..= 1.0

fn main():
    r: Roughness = 0.5
    big = r * 4.0
    back: Roughness = big     #$ error[E2215]: a `Roughness` cannot be built from a value this is not known to be in range
    v: f32 = back
    println(v)
