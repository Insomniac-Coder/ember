#$ test: compile-fail
#$ rules: RNG-1
## "an `in` clause on a non-numeric representation" — IV.2a's `E2213`.

type Odd = bool in 0 ..= 1   #$ error[E2213]: `bool` is not a numeric type
fn main(): pass
