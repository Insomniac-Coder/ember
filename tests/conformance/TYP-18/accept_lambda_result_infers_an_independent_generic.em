#$ test: run-pass
#$ rules: TYP-18, TYP-23, CLO-1, CLO-3, FN-6a, MONO-1
#$ stdout: 5

fn apply[T, R](value: T, transform: fn(T) -> R) -> R:
    return transform(value)

fn main():
    println(apply(4, fn(item) => item + 1))
