#$ test: compile-fail
#$ rules: RNG-5
## "Arithmetic between two distinct nominal range types is rejected (`E2214`)
## unless at least one operand is explicitly converted to its representation."

type Roughness = f32 in 0.0 ..= 1.0
type Metallic  = f32 in 0.0 ..= 1.0

fn main():
    r: Roughness = 0.5
    m: Metallic = 0.5
    bad = r + m       #$ error[E2214]: `+` is not defined between `Roughness` and `Metallic`
    print(1)
