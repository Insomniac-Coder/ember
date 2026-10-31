#$ test: run-pass
#$ rules: TYP-5, TYP-18, MONO-1

[T]
fn identity(value: T) -> T:
    return value

fn main():
    println(identity[i64](53))
#$ stdout: 53
