#$ test: compile-fail
#$ rules: RNG-1
## "…whose endpoints are constant expressions of the representation type" —
## and not inverted.

type Backwards = f32 in 1.0 ..= 0.0   #$ error[E2212]: the range is inverted
fn main(): pass
