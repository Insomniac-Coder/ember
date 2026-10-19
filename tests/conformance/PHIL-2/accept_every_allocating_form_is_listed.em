#$ test: run-pass
#$ rules: PHIL-2, CLO-10, TXT-9
#$ profiles: debug, development
#$ stdout: 1 4 5 1 2 2 5 6 3 2 6 1 2 10 20
# `[PHIL-2]` — every allocating form the source can show, each listed by `ember inspect --alloc`
# (the milestone `inspect_alloc_lists_every_allocating_form` reads this program's report, by
# line): a class instance, an `Array`, a `String`, a `Map`, a `Set`, a `Box`, a `Shared`; a list,
# map and set literal and a comprehension; a string literal that becomes a `String`; an f-string;
# an owned callable value holding more than three words of captured state (`[CLO-10]`); and a
# collection's growth. `total` allocates nothing and is not listed.

class Node:
    value: int

fn four(a: int, b: int, c: int, d: int) -> fn() -> int:
    return owned fn() => a + b + c + d

fn three(a: int, b: int, c: int) -> fn() -> int:
    return owned fn() => a + b + c

fn total(xs: Span[int]) -> int:
    sum = 0
    for x in xs:
        sum += x
    return sum

fn main():
    node = Node(1)
    empty = Array[int]()
    buf = String()
    xs = [1, 2, 3]
    xs.push(4)
    name: String = "ember"
    text = f"node {node.value}"
    ages: Map[str, int] = Map()
    ages["ada"] = 36
    seen: Set[int] = {1, 2}
    boxed = Box(5)
    shared = Shared(6)
    doubled = [x * 2 for x in xs]
    pairs = {"a": 1, "b": 2}
    odd = {x for x in xs if x % 2 == 1}
    many = four(1, 2, 3, 4)
    few = three(1, 2, 3)
    println(node.value, xs.len(), name.len(), ages.len(), seen.len(), pairs.len(), boxed.get(), shared.get(), empty.len() + 3, odd.len(), few(), text.len() - 5, buf.len() + 2, many(), total(doubled))
