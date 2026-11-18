#$ test: run-pass
#$ rules: TYP-36, EXP-9
#$ profiles: debug, development, release
#$ stdout: true false true 5
#$ hi hi
#$ 7 true
#$ [3, 1, 2]
#$ 5
#$ true false
#$ true 1
#$ 9
#$ [1, 2, 3]
#$ true false
# `[TYP-36]` — a `shares T`, `shares mut T` or `thread_shared T` owner implements `Eq`, `Ord`, `Hash`,
# `Display` and `Debug` as `T` does, on the value it owns: two owners compare their values, and whether
# they own one value is `a is b` (`[EXP-9]`). A class object's `==` stays its identity.

class Node:
    v: int = 1

[T: needs Display]
fn show(x: T):
    println(x)

[T: needs Eq]
fn same(a: T, b: T) -> bool:
    return a == b

[T: needs Ord]
fn lower(a: T, b: T) -> bool:
    return a < b

fn main():
    a = shares 5
    b = shares 5
    c = shares a
    println(a == b, a is b, a is c, a)
    d = shares mut "hi".to_string()
    println(d, f"{d}")
    t = thread_shared 7
    println(t, t == t)
    xs = [shares 3, shares 1, shares 2]
    println(xs)
    show(a)
    println(same(a, b), lower(a, b))
    keys: Set[shares int] = Set()
    k = shares 1
    keys.add(shares k)
    println(keys.contains(k), len(keys))
    m: Map[shares int, int] = Map()
    _none = m.insert(shares k, owns 9)
    println(m[k])
    xs.sort()
    println(xs)
    n1 = shares Node()
    n2 = shares n1
    n3 = shares Node()
    println(n1 == n2, n1 == n3)
