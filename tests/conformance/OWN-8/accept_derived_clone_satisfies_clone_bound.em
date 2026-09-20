#$ test: run-pass
#$ rules: OWN-8, TYP-17
#$ profiles: debug, release, shipping
#$ stdout: 42
#$ stdout: 42

@derive(Clone)
struct Pair:
    value: i32

fn duplicate[T: Clone](value: T) -> T:
    return value.clone()

fn main():
    original = Pair(42)
    copied = duplicate(original)
    println(original.value)
    println(copied.value)
