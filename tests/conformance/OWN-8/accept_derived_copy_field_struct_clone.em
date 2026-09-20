#$ test: run-pass
#$ rules: OWN-8
#$ profiles: debug, release, shipping
#$ stdout: 42
#$ stdout: 42
#$ assert-c: contains(em_Pair_clone)

@derive(Clone)
struct Pair:
    value: i32

fn main():
    original = Pair(42)
    copied = original.clone()
    println(original.value)
    println(copied.value)
