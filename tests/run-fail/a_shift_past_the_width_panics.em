#$ test: run-fail
#$ rules: TYP-10
#$ panics: integer overflow
fn main():
    a: u32 = 1
    n: u32 = 32
    println(a << n)
