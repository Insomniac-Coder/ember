#$ test: compile-fail
#$ rules: CT-1, TYP-1
# V.7 — a `const` is copied into each use, so its type is `Copy` and owns no
# heap memory: a class handle or a struct holding an `Array` is `E2130`,
# whose help is a `static`. A value this phase cannot work out while
# compiling, a function's result, is `E1010`.

class Node:
    x: int

struct Heavy:
    items: Array[int]

fn three() -> int:
    return 3

const A: Node = Node(1)    #$ error[E2130]: a `const` cannot hold a `Node`
const B: Heavy = Heavy([])    #$ error[E2130]: a `const` cannot hold a `Heavy`
const C: int = three() + 1    #$ error[E1010]: a `const` is a literal, another constant
const D = String.from("x")    #$ error[E2130]: a `const` cannot hold a `String`

fn main():
    println(A.x, C)
