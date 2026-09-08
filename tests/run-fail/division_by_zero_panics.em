#$ test: run-fail
#$ rules: TYP-8
#$ panics: division by zero
fn main():
    n: i32 = 10
    d: i32 = 0
    println(n / d)
