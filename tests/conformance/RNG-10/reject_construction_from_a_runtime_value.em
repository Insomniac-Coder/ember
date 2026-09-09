#$ test: compile-fail
#$ rules: RNG-10
## "Constructing a range-typed value outside this set in Safe code is `E2215`."

type Percent = u8 in 0 ..= 100

fn from_input(x: u8) -> Percent:
    return x        #$ error[E2215]: a `Percent` cannot be built
fn main(): pass
