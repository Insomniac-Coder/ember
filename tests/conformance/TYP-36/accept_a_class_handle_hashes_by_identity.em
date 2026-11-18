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
    seen: Set[shares Node] = Set()
    seen.add(shares a)
    seen.add(shares b)
    seen.add(shares a)
    other = shares Node("a")
    println(len(seen), seen.contains(shares a), seen.contains(other))
    m: Map[shares Node, int] = Map()
    m[shares a] = 1
    m[shares b] = 2
    m[shares a] = 3
    println(m[shares a], m[shares b], len(m))
