#$ test: run-pass
#$ rules: OWN-8, TYP-16, TYP-17
#$ profiles: debug, release, shipping
#$ stdout: 42
#$ stdout: 42

@derive(Clone)
struct Wrapper[T]:
    value: T

fn duplicate[T: Clone](value: T) -> T:
    return value.clone()

fn main():
    original = Wrapper(42)
    copied = duplicate(original)
    println(original.value)
    println(copied.value)
