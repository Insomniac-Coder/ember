#$ test: run-pass
#$ rules: TYP-18, TYP-23, CLO-2, CLO-3, FN-6a, MONO-1
#$ stdout: 7

fn apply[T, R](value: T, transform: fn(T) -> R) -> R:
    return transform(value)

fn main():
    offset = 3
    println(apply(4, fn(item) => item + offset))
