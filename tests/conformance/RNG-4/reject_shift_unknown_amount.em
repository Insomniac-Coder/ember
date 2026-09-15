#$ test: compile-fail
#$ rules: RNG-4, RNG-10, TYP-10

type Positive = i32 in 0 ..= 100
type Shifted = i32 in 0 ..= 200

fn left(value: Positive, amount: i32) -> Shifted:
    return value << amount    #$ error[E2215]
