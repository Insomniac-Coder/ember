#$ test: run-pass
#$ rules: TYP-18, TYP-23, CLO-3, MONO-1

fn apply[T](transform: fn(T) -> T, value: T) -> T:
    return transform(value)

fn main():
    println(apply(fn(value) => value, 54))
#$ stdout: 54
