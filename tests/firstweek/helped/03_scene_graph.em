# A scene graph: nodes with local offsets, children and weak parent links, printing each node's world position.

from std.math import Vec3

class Node:
    name: String
    local: Vec3
    children: Array[shares mut Node] = []
    parent: weakref Node = None

    fn init(owns name: String, local: Vec3):
        this.name = name
        this.local = local

    fn add_child(owns child: shares mut Node):
        child.parent = weakref this
        this.children.push(owns child)

    fn world_position() -> Vec3:
        pos = this.local
        up = weakref this.parent
        while up exists as p:
            pos = pos + p.local
            up = weakref p.parent
        return pos

    fn depth() -> int:
        d = 0
        up = weakref this.parent
        while up exists as p:
            d += 1
            up = weakref p.parent
        return d

    fn path() -> String:
        if this.parent exists as p:
            return f"{p.path()}/{this.name}"
        return copy of this.name

fn print_tree(node: Node):
    indent = "  ".repeat(node.depth())
    w = node.world_position()
    l = node.local
    println(f"{indent}{node.name}: local ({l.x}, {l.y}, {l.z}) world ({w.x}, {w.y}, {w.z})")
    for child in node.children:
        print_tree(child)

fn find(owns node: shares mut Node, name: StrView) -> Option[shares mut Node]:
    if node.name == name:
        return Some(node)
    for child in node.children:
        if find(shares mut child, name) exists as found:
            return Some(shares mut found)
    return None

fn count_nodes(node: Node) -> int:
    return 1 + sum(count_nodes(c) for c in node.children)

fn main():
    root = shares mut Node("world", Vec3(0, 0, 0))
    house = shares mut Node("house", Vec3(10, 0, 5))
    roof = shares mut Node("roof", Vec3(0, 3, 0))
    chimney = shares mut Node("chimney", Vec3(1, 1, 0))
    tree = shares mut Node("tree", Vec3(-4, 0, 2))
    car = shares mut Node("car", Vec3(3, 0, -2))
    wheel = shares mut Node("wheel", Vec3(0.5, -0.3, 0))
    root.add_child(shares mut house)
    house.add_child(shares mut roof)
    roof.add_child(shares mut chimney)
    root.add_child(shares mut tree)
    root.add_child(shares mut car)
    car.add_child(shares mut wheel)
    print_tree(root)
    println(f"{count_nodes(root)} nodes")

    if find(shares mut root, "chimney") exists as n:
        println(f"path of chimney: {n.path()}")

    # move the house: the roof and the chimney move with it
    house.local = house.local + Vec3(0, 0, 10)
    if find(shares mut root, "chimney") exists as n:
        w = n.world_position()
        println(f"chimney after the move: ({w.x}, {w.y}, {w.z})")

    if find(shares mut root, "garage") is None:
        println("no garage in this scene")
