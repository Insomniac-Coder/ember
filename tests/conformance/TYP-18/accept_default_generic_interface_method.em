#$ test: run-pass
#$ rules: TYP-17, TYP-18, MONO-1

interface Identity:
    fn keep[T](self, value: T) -> T:
        return value

struct UsesDefault implements Identity:
    marker: i32

fn main():
    value = UsesDefault(0)
    println(value.keep(50))
#$ stdout: 50
