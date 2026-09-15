#$ test: compile-fail
#$ profiles: debug, release, shipping
#$ error[E2020]: `is` requires related class handles, found `an integer` and `an integer`

fn main():
    result = 1 is 1
    println(result)
