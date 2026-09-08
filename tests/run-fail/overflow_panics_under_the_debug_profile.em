#$ test: run-fail
#$ rules: TYP-8
#$ panics: integer overflow
fn main():
    a: i32 = 2147483647
    b: i32 = 1
    println(a + b)
