#$ test: run-pass
#$ rules: TYP-16, TYP-18, MONO-1

[T]
struct Holder:
    value: T

    [U]
    static fn keep(value: U) -> U:
        return value

fn main():
    println(Holder[i32].keep(56))
    println(Holder[i64].keep[i64](57))
#$ stdout: 56
#$ 57
