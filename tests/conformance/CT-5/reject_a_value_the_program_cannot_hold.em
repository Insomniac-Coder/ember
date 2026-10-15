#$ test: compile-fail
#$ rules: CT-5, V.7
#$ error[E6010]: a compile-time value cannot be a `Node`
#$ error[E2130]: a `const` cannot hold a `Array[i64]`
# `[CT-5]` — a compile-time value is placed in the program as constant data, so it cannot be a
# class handle (or any view into the evaluator's memory); and a `const` cannot own heap memory.

class Node:
    x: int

fn node() -> Node:
    return Node(1)

fn list() -> Array[int]:
    return [1, 2]

const LIST: Array[int] = list()

fn main():
    n = comptime(node())
    println(n.x, LIST)
