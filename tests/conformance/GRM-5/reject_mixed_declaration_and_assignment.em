#$ test: compile-fail
#$ rules: GRM-5
#$ error[E1010]: all names in a destructuring assignment must be declared or assigned consistently

fn main():
    existing = 1
    existing, fresh = (2, 3)

