#$ test: compile-fail
#$ rules: XVIII.4.6
#$ error[E3050]: may not have a value here
fn main():
    n: i32 = 3
    x: i32
    if n > 0:
        x = 1
    println(x)
