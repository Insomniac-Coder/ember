#$ test: run-pass
#$ rules: OWN-8
#$ profiles: debug, release, shipping
#$ stdout: 42
#$ stdout: 42
#$ assert-c: contains(em_Leaf_clone)
#$ assert-c: contains(em_Pair_clone)

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
