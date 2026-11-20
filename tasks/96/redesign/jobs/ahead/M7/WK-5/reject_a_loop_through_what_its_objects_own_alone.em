# Clause WK-5.1: "every place a type owns alone (a plain field or element, an `owns T`, a struct or
# enum payload stored inline), to the type held there, since what a value owns alone reaches what
# that owns"
#$ test: compile-fail
#$ rules: WK-5, WK-6, WK-1
# The owners sit in a struct each node owns alone. No text in the spec for this program: code only.

struct Holder:
    n: Option[shares mut Node] = None

class Node:
    name: String
    h: Holder = Holder()

fn main():
    a = shares mut Node(name=String("a"))
    b = shares mut Node(name=String("b"))
    a.h.n = Some(shares mut b)
    b.h.n = Some(shares mut a)    #$ error[E3032]
