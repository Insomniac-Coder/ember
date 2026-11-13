# Clause CTL-11.1: "An `if` or `while` header that opens an `Option` or a `weakref` link (`if
# cam.target exists as p:`, `while cur.parent exists as p:`, `[GRM-44]`), and an `if` or `elif`
# header that tests a value's kind (`if s is Circle as c:`, `[DSP-4]`), run their block as those
# rules say"
#$ test: run-pass
#$ rules: CTL-11, GRM-44, DSP-4, WK-11
#$ stdout: 3
#$ stdout: c
#$ stdout: b
#$ stdout: a
#$ stdout: square 2

interface Shape:
    fn area() -> int

struct Circle:
    r: int

struct Square:
    side: int

extend Circle implements Shape:
    fn area() -> int:
        return 3 * this.r * this.r

extend Square implements Shape:
    fn area() -> int:
        return this.side * this.side

class Node:
    name: String
    parent: weakref Node = None

fn kind(s: dyn Shape) -> String:
    if s is Circle as c:
        return f"circle {c.r}"
    elif s is Square as q:
        return f"square {q.side}"
    return String("other")

fn main():
    o: Option[int] = Some(3)
    if o exists as v:
        println(v)
    a = Node(name=String("a"))
    b = Node(name=String("b"))
    c = Node(name=String("c"))
    b.parent = weakref a
    c.parent = weakref b
    println(c.name)
    cur = ref c
    while cur.parent exists as p:
        println(p.name)
        cur = p
    println(kind(Square(side=2)))
