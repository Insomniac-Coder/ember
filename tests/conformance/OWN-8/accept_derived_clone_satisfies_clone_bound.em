#$ test: run-pass
#$ rules: OWN-8, TYP-17
#$ profiles: debug, development, release
#$ stdout: 42
#$ stdout: 42

@derive(Clone)
struct Pair:
    value: i32

[T: needs Clone]
fn duplicate(value: T) -> T:
    return value.clone()

fn main():
    original = Pair(42)
    copied = duplicate(original)
    println(original.value)
    println(copied.value)
