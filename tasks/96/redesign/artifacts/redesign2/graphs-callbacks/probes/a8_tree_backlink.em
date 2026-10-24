# A8, proposed: `parent` is the back link of `children`; the language keeps the two in step.
# Today: an error (the spelling does not exist). The report gives the rules.
class Node:
    name: String
    parent: Option[Node] = backlink(children)
    children: Array[Node] = []

fn path(n: Node) -> String:
    match n.parent:
        Some(p) => return f"{path(p)}/{n.name}"
        None => return n.name.clone()

fn main():
    root = Node("root")
    leaf = Node("leaf")
    root.children.push(leaf)          # also sets leaf.parent
    other = Node("other")
    other.children.push(leaf)         # moves leaf: out of root.children, parent = other
    println(path(leaf), root.children.len())
