#$ test: run-pass
#$ rules: OWN-8, TYP-16, TYP-17
#$ profiles: debug, development, release
#$ stdout: 42
#$ stdout: 42

@derive(Clone)
[T]
struct Wrapper:
    value: T

[T: needs Clone]
fn duplicate(value: T) -> T:
    return value.clone()

fn main():
    original = Wrapper(42)
    copied = duplicate(original)
    println(original.value)
    println(copied.value)
