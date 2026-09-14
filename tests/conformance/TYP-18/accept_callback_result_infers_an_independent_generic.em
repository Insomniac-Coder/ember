#$ test: run-pass
#$ rules: TYP-18, TYP-23, CLO-3, FN-6a, MONO-1
#$ stdout: 5
#$ true

# The callback is the only argument that mentions `R`; inference must read the
# solved callable signature instead of solving only the hidden Callable binder.

fn apply[T, R](value: T, transform: fn(T) -> R) -> R:
    return transform(value)

fn increment(value: i32) -> i32:
    return value + 1

fn positive(value: i32) -> bool:
    return value > 0

fn main():
    println(apply(4, increment))
    println(apply(4, positive))
