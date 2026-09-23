#$ test: compile-fail
#$ rules: STD-8
#$ profiles: debug
#$ error[E2226]: `i64` does not implement `Contains`

fn main():
    x = 5
    println(1 in x)
