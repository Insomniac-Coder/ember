#$ test: run-pass
#$ rules: OWN-8, MOD-5, TYP-17
#$ profiles: debug, release, shipping
#$ stdout: 42

struct Leaf implements Clone:
    value: i32

    fn clone(self) -> Self:
        return Leaf(self.value)

fn duplicate[T: Clone](value: T) -> T:
    return value.clone()

fn main():
    println(duplicate(Leaf(42)).value)
