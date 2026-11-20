# Clause CLO-17.2: "since the run would end or move the function's own state (`b.on_click(owns
# fn(): b.handlers.clear())`, `t.on_visit = fn(n: Node): t.root.children.clear()`, (...) `E3022`"
#$ test: compile-fail
#$ rules: CLO-17
# The clause's second example, alone because it depends on Q3: by the clause's letter `t.on_visit`
# is not a place `t.root.children.clear()` reshapes or ends, nor owned by one; the example says
# `E3022`. This file follows the example. No text in the spec: code only.

class Node:
    name: String
    children: Array[Node] = []

class Tree:
    root: Node
    on_visit: fn(Node) = look

    fn init(owns root: Node):
        this.root = root

fn look(n: Node):
    println(n.name)

fn main():
    t = Tree(Node("root"))
    t.on_visit = fn(n: Node): t.root.children.clear()    #$ error[E3022]
    t.on_visit(t.root)
