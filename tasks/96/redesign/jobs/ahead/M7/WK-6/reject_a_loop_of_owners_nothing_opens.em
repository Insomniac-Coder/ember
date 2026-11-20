# Clause WK-6.1: "A line that closes a loop of owners (`a.next = Some(shares mut b)` after `b.next`
# names `a`) is refused, unless the compiler proves that the loop is opened again before its objects
# lose their last owner from outside (`E3032`"
#$ test: compile-fail
#$ rules: WK-6, WK-1, WK-5
# The spec's own program and message.

class Node:
    name: String
    next: Option[shares mut Node] = None

fn make_pair():
    a = shares mut Node(name=String("a"))
    b = shares mut Node(name=String("b"))
    a.next = Some(shares mut b)
    b.next = Some(shares mut a)    #$ error[E3032]: this closes a loop of owners (`a` → `b` → `a`) that nothing opens again before `a` and `b` end, so neither node is ever freed; open it before the function ends: `b.next = None`, or make the link back `weakref`: `next: weakref Node = None`

fn main():
    make_pair()
