#$ test: run-pass
#$ rules: OWN-8
#$ profiles: debug, development, release
#$ stdout: 42
#$ stdout: 42
#$ assert-c: contains(em_4Leaf5clone)
#$ assert-c: contains(em_4Pair5clone)

struct Leaf:
    value: i32

    fn clone(self) -> Leaf:
        return Leaf(self.value)

@derive(Clone)
struct Pair:
    leaf: Leaf

fn main():
    original = Pair(Leaf(42))
    copied = original.clone()
    println(original.leaf.value)
    println(copied.leaf.value)
