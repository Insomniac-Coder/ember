#$ test: run-pass
#$ rules: TYP-14, TYP-16
#$ stdout: 6
#$ stdout: 2
#$ stdout: 1 2

## D-731 — a generic struct, enum or class names itself in its own fields, through a `Box` or an
## `Array`. Each was `E1010`, "cannot find type `Node`": a generic type was registered only once its
## fields were resolved. It is registered first now, and an instance named meanwhile is filled once
## the declaration is whole.

[T]
struct Node:
    value: T
    next: Option[owns Node[T]]

enum List[T]:
    Nil
    Cons(T, owns List[T])

[T]
class Tree:
    value: T
    kids: Array[Weak[Tree[T]]]

    fn init(value: T):
        this.value = value
        this.kids = Array[Weak[Tree[T]]]()

fn node_sum(n: Node[int]) -> int:
    match n.next:
        Some(rest):
            return n.value + node_sum(rest)
        None:
            return n.value

fn length(l: List[int]) -> int:
    match l:
        Nil:
            return 0
        Cons(_, rest):
            return 1 + length(rest)

fn main():
    last = Node[int](value=3, next=None)
    middle = Node[int](value=2, next=Some(owns last))
    println(node_sum(Node[int](value=1, next=Some(owns middle))))
    println(length(List[int].Cons(1, owns List[int].Cons(2, owns List[int].Nil))))
    root = Tree[int](1)
    kid = Tree[int](2)
    root.kids.push(owns Weak(kid))
    println(root.value, root.kids[0].upgrade().unwrap().value)
