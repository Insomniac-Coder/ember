#$ test: run-fail
#$ rules: TXT-11, TXT-4, TYP-31, HEAP-8
#$ profiles: debug, release, shipping
#$ panics: String size or byte offset is negative or too large

fn main():
    n: i128 = 18446744073709551616
    s = String.with_capacity(n)
    println(s)
