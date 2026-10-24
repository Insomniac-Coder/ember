# A8: the tree a Python programmer writes. The parent link is a plain field; `add` sets both ends.
class Node:
    name: String
    children: Array[Node] = []
    parent: Option[Node] = None

    fn add(self, child: Node):
        child.parent = Some(self)
        self.children.push(child)

fn path(n: Node) -> String:
    match n.parent:
        Some(p) => return f"{path(p)}/{n.name}"
        None => return n.name.clone()

fn main():
    root = Node("root")
    leaf = Node("leaf")
    root.add(leaf)
    println(path(leaf))
