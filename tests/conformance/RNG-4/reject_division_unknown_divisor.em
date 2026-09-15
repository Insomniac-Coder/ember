#$ test: compile-fail
#$ rules: RNG-4, RNG-10

type Wide = i32 in -100 ..= 100
type Narrow = i32 in -10 ..= 10

fn quotient(value: Wide, divisor: i32) -> Narrow:
    return value / divisor    #$ error[E2215]
