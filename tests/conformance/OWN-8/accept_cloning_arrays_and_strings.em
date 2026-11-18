#$ test: run-pass
#$ rules: OWN-8, STR-5
#$ stdout: [1, 2] [1, 2, 3]
#$ hi hi!
#$ ['a', 'b'] ['a', 'b', 'c']
#$ [[1], [2, 3]] [[1, 9], [2, 3]]
#$ 2 1 2
#$ ['x'] ['x', 'y']
#$ q q! ['p']
# `[OWN-8]` — `clone` is the explicit deep copy. An `Array` clones each
# element (a nested array, a `String`, a struct through its `clone`, a class
# handle by copying the handle), a `String` copies its bytes, and a derived
# `Clone` clones such fields. Every clone is independent of its original.

@derive(Clone)
class Token:
    n: int

@derive(Clone)
struct Bag:
    items: Array[String]
    name: String

fn main():
    xs: Array[int] = [1, 2]
    ys = copy of xs
    ys.push(owns 3)
    println(xs, ys)
    s: String = "hi"
    t = copy of s
    t.push_str("!")
    println(s, t)
    words: Array[String] = ["a", "b"]
    copy = copy of words
    copy.push(owns "c")
    println(words, copy)
    grid: Array[Array[int]] = [[1], [2, 3]]
    g2 = copy of grid
    g2[0].push(owns 9)
    println(grid, g2)
    tokens: Array[Token] = [Token(1), Token(2)]
    t2 = copy of tokens
    println(t2.len(), tokens[0].n, t2[1].n)
    bags: Array[Bag] = [Bag(items=["x"], name="b")]
    b2 = copy of bags
    b2[0].items.push(owns "y")
    println(bags[0].items, b2[0].items)
    bag = Bag(items=["p"], name="q")
    bag2 = copy of bag
    bag2.name.push_str("!")
    println(bag.name, bag2.name, bag2.items)
