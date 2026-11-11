#$ test: compile-fail
#$ rules: EXP-6, EXC-1
# D-302, `[OWN-11]` — moving out of a class field is `E3013` (`E3012` is folded into it): the object keeps the field
# and every handle to it can still reach it, so `b.clear()` would free the
# `String` that `x` holds. It compiled, and the program read freed memory.
# (A `match` on a class field binds by reference instead, its loan holding
# the field's read access: GRM-13, D-459.)

class Node:
    name: Option[String]

    fn clear():
        this.name = None

fn take(a: Node) -> Option[String]:
    return a.name    #$ error[E3013]: `a` is a `Node` object, which other names may reach, so `a.name` cannot be taken out and leave a hole; take it and leave a value in its place: `mem.take(ref mut a.name)`

fn keep(a: Node):
    x = a.name    #$ error[E3013]: `a` is a `Node` object, which other names may reach, so `a.name` cannot be taken out and leave a hole; take it and leave a value in its place: `mem.take(ref mut a.name)`
    println(x)

fn main():
    a = Node(name=Some("hello"))
    keep(a)
    println(take(a))
