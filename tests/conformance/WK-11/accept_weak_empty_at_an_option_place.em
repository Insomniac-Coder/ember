#$ test: run-pass
#$ rules: WK-11, TYP-5
#$ stdout: true
# D-434 — at an `Option[Weak[Node]]` place `[TYP-5]` rule 11 makes
# `Weak.empty()` a `Some` of the `Weak[Node]` the place holds, so its type is
# known. It was refused, "cannot infer which `Weak` this is".

class Node:
    v: int

fn main():
    w: Option[Weak[Node]] = Weak.empty()
    match w:
        Some(x) => println(x.upgrade().is_none())
        None => println("none")
