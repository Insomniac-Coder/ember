#$ test: compile-fail
#$ rules: RNG-5a1
## "No `*Assign` form is generated: `r += 1.0` would produce an `R` where a `T`
## is required and is `E2214`, whose help names `r = Roughness.clamped(r + 0.1)`."

type Roughness = f32 in 0.0 ..= 1.0

fn main():
    r: Roughness = 0.5
    r += 0.1          #$ error[E2214]: `+=` is not defined on `Roughness`
    print(1)
