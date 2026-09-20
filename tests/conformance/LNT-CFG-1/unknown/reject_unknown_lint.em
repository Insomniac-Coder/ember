#$ test: compile-fail
#$ rules: MAN-3
#$ error[E9010]: is not a lint the compiler defines

fn main():
    println(42)
