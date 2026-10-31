#$ test: run-pass
#$ rules: OWN-8, MOD-5, TYP-17
#$ profiles: debug, development, release
#$ stdout: 42

struct Leaf implements Clone:
    value: i32

    fn clone() -> Self:
        return Leaf(this.value)

[T: needs Clone]
fn duplicate(value: T) -> T:
    return value.clone()

fn main():
    println(duplicate(Leaf(42)).value)
