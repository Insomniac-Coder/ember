#$ test: compile-fail
#$ profiles: debug, release, shipping
#$ error[E2150]: `is` needs class handles, and `an integer` is not one

fn main():
    result = 1 is 1
    println(result)
