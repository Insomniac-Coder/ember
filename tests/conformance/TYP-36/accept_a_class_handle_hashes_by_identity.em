#$ test: run-pass
#$ rules: TYP-36, STD-11
#$ stdout: 2 true false
#$ stdout: 3 2 2
# `[TYP-36]` — a class handle is `Hash` by identity, as its `Eq` is identity
# (`is`): a handle can key a `Map` or join a `Set`, and another object with
# the same fields is another key. D-273: `K: Hash` refused a class handle.

class Node:
    name: String

fn main():
    a = Node("a")
    b = Node("b")
    seen: Set[Node] = Set()
    seen.add(a)
    seen.add(b)
    seen.add(a)
    other = Node("a")
    println(len(seen), seen.contains(a), seen.contains(other))
    m: Map[Node, int] = Map()
    m[a] = 1
    m[b] = 2
    m[a] = 3
    println(m[a], m[b], len(m))
