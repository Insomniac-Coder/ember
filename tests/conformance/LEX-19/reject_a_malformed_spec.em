#$ test: compile-fail
#$ rules: LEX-19
#$ profiles: debug
#$ error[E0100]: `:>>q` is not a format spec

fn main():
    n = 5
    println(f"{n:>>q}")
