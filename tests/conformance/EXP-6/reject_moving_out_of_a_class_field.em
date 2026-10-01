#$ test: compile-fail
#$ rules: EXP-6, EXC-1
# D-302 — moving out of a class field is `E3012`: the object keeps the field
# and every handle to it can still reach it, so `b.clear()` would free the
# `String` that `x` holds. It compiled, and the program read freed memory.
# (A `match` on a class field binds by reference instead, its loan holding
# the field's read access: GRM-13, D-459.)

class Node:
    name: Option[String]

    fn clear(mut self):
        self.name = None

fn take(a: Node) -> Option[String]:
    return a.name    #$ error[E3012]: cannot move out of a class field

fn keep(a: Node):
    x = a.name    #$ error[E3012]: cannot move out of a class field
    println(x)

fn main():
    a = Node(name=Some("hello"))
    keep(a)
    println(take(a))
