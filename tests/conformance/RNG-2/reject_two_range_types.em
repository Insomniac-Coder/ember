#$ test: compile-fail
#$ rules: RNG-2
## "Two range types are distinct types even when representation and range are
## identical" — IV.2a's own worked example.

type Roughness = f32 in 0.0 ..= 1.0
type Metallic  = f32 in 0.0 ..= 1.0

fn main():
    r: Roughness = 0.5
    m: Metallic  = r        #$ error[E2210]: `Roughness` is not `Metallic`
    print(1)
