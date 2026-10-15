#$ test: compile-fail
#$ rules: CT-1, TYP-1
# V.7 — a `const` is copied into each use, so its type is `Copy` and owns no
# heap memory: a class handle or a struct holding an `Array` is `E2130`,
# whose help is a `static`. A function's result is worked out while
# compiling (`[CT-1]`), so `C` is a constant.

class Node:
    x: int

struct Heavy:
    items: Array[int]

fn three() -> int:
    return 3

const A: Node = Node(1)    #$ error[E2130]: a `const` cannot hold a `Node`
const B: Heavy = Heavy([])    #$ error[E2130]: a `const` cannot hold a `Heavy`
const C: int = three() + 1
const D = String.from("x")    #$ error[E2130]: a `const` cannot hold a `String`

fn main():
    println(A.x, C)
