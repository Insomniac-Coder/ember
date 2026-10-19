#$ test: compile-fail
#$ rules: TYP-14
#$ error[E2020]: expected `Tree`, found `Box[Tree]`
# `[TYP-14]` reads a `Box` through where a value is read, not where one is moved out: a local
# declared `Tree` would take the payload out of its box. D-732 — a diagnostic naming an enum that
# holds itself through a `Box` never ended (the compiler's stack overflowed).

enum Tree:
    Leaf
    Node(Box[Tree])

fn main():
    b = Box(Tree.Leaf)
    t: Tree = b
