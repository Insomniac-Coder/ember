#$ test: compile-fail
#$ rules: XVIII.4.6
#$ error[E3050]: is used before it is given a value
fn main():
    x: i32
    println(x)
