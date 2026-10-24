# A8: moving a node to a new parent with today's Weak spelling (Python writes three plain lines).
class Node:
    value: int
    children: Array[Node] = []
    parent: Weak[Node] = Weak.empty()

fn reparent(node: Node, new_parent: Node):
    match node.parent.upgrade():
        Some(old):
            i = old.children.index_of(node).unwrap()
            _gone = old.children.remove(i)
        None:
            pass
    node.parent = Weak(new_parent)
    new_parent.children.push(node)

fn main():
    a = Node(1)
    b = Node(2)
    c = Node(3)
    a.children.push(c)
    c.parent = Weak(a)
    reparent(c, b)
    println(b.children.len())
