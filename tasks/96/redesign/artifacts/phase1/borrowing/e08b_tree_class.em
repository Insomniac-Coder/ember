# A8 with classes and a Weak parent (the spec's Part VIII example, plus a path walk).
class Node:
    name: String
    children: Array[Node] = []
    parent: Weak[Node] = Weak.empty()

    fn add(self, child: Node):
        child.parent = Weak(self)
        self.children.push(child)

fn path(n: Node) -> String:
    match n.parent.upgrade():
        Some(p) => return f"{path(p)}/{n.name}"
        None => return n.name.clone()

fn main():
    root = Node("root")
    leaf = Node("leaf")
    root.add(leaf)
    println(path(leaf))
