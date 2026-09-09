#$ test: compile-fail
#$ rules: RNG-1
## An endpoint that is not a constant of the representation type.

fn width() -> f32: return 1.0
type Bad = f32 in 0.0 ..= width()   #$ error[E2212]: a range endpoint must be a constant
fn main(): pass
