#$ test: compile-fail
#$ rules: RNG-4, RNG-10
#$ profiles: debug, release, shipping

type Small = i32 in 0 ..= 15

fn not_proven(value: i32) -> Small:
    return value | 15  #$ error[E2215]
