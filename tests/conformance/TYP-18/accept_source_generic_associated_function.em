#$ test: run-pass
#$ rules: TYP-16, TYP-18, MONO-1

struct Factory:
    marker: i32

    fn identity[T](value: T) -> T:
        return value

fn main():
    println(Factory.identity(45))
    println(Factory.identity[i64](46i64))
#$ stdout: 45
#$ 46
