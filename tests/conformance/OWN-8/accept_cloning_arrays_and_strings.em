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

class Token:
    n: int

@derive(Clone)
struct Bag:
    items: Array[String]
    name: String

fn main():
    xs: Array[int] = [1, 2]
    ys = xs.clone()
    ys.push(3)
    println(xs, ys)
    s: String = "hi"
    t = s.clone()
    t.push_str("!")
    println(s, t)
    words: Array[String] = ["a", "b"]
    copy = words.clone()
    copy.push("c")
    println(words, copy)
    grid: Array[Array[int]] = [[1], [2, 3]]
    g2 = grid.clone()
    g2[0].push(9)
    println(grid, g2)
    tokens: Array[Token] = [Token(1), Token(2)]
    t2 = tokens.clone()
    println(t2.len(), tokens[0].n, t2[1].n)
    bags: Array[Bag] = [Bag(items=["x"], name="b")]
    b2 = bags.clone()
    b2[0].items.push("y")
    println(bags[0].items, b2[0].items)
    bag = Bag(items=["p"], name="q")
    bag2 = bag.clone()
    bag2.name.push_str("!")
    println(bag.name, bag2.name, bag2.items)
