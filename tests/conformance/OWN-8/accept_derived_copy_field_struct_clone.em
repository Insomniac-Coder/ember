#$ test: run-pass
#$ rules: OWN-8
#$ profiles: debug, development, release
#$ stdout: 42
#$ stdout: 42
#$ assert-c: contains(em_4Pair5clone)

@derive(Clone)
struct Pair:
    value: i32

fn main():
    original = Pair(42)
    copied = original.clone()
    println(original.value)
    println(copied.value)
