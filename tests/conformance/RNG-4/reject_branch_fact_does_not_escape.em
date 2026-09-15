#$ test: compile-fail
#$ rules: RNG-4, RNG-10

type Percent = i32 in 0 ..= 100

fn invalid_after_branch(value: i32):
    if value >= 0 and value <= 100:
        pass
    # The comparison fact is valid only in the then arm.
    result: Percent = value    #$ error[E2215]
