#$ test: compile-fail
#$ rules: EXP-6, EXC-1
# D-302 — moving out of a class field is `E3012`: the object keeps the field
# and every handle to it can still reach it, so `b.clear()` would free the
# `String` that `x` (or the match's `s`) holds. It compiled, and the program
# read freed memory. A match on a class field binds by move, not by
# reference: a reference would need a read access the handle's aliases
# respect, which is not built (D-202).

class Node:
    name: Option[String]

    fn clear(mut self):
        self.name = None

fn take(a: Node) -> Option[String]:
    return a.name    #$ error[E3012]: cannot move out of a class field

fn inspect(a: Node):
    b = a
    match a.name:
        Some(s):    #$ error[E3012]: cannot move out of a class field
            b.clear()
            println(s)
        None:
            pass

fn keep(a: Node):
    x = a.name    #$ error[E3012]: cannot move out of a class field
    println(x)

fn main():
    a = Node(name=Some("hello"))
    keep(a)
    inspect(a)
    println(take(a))
