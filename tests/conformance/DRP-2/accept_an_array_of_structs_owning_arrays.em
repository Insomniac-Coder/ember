#$ test: run-pass
#$ rules: DRP-2
#$ stdout: 2
# D-231 — dropping an `Array` whose elements own an `Array` of values that
# need dropping loops inside a loop; each level has its own index.

struct Bag:
    items: Array[String]
    name: String

fn main():
    bags: Array[Bag] = [Bag(items=["x", "y"], name="a"), Bag(items=["z"], name="b")]
    println(bags.len())
