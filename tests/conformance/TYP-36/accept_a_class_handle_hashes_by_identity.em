#$ test: run-pass
#$ rules: TYP-36, STD-11
#$ stdout: false
#$ stdout: 2 false
#$ stdout: 1
# `[TYP-36]` — a class handle is `Hash` by identity, as its `Eq` is identity
# (`is`): a handle can key a `Map` or join a `Set`, and another object with
# the same fields is another key. D-273: `K: Hash` refused a class handle.

class Node:
    name: String

fn main():
    a = Node("a")
    b = Node("b")
    other = Node("a")
    seen: Set[Node] = Set()
    println(seen.contains(a))
    seen.add(owns a)
    seen.add(owns b)
    println(len(seen), seen.contains(other))
    m: Map[Node, int] = Map()
    _none = m.insert(owns other, owns 3)
    println(len(m))
