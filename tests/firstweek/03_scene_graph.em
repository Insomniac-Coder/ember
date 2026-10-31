# A scene graph: nodes with local offsets, children and weak parent links, printing each node's world position.

from std.math import Vec3

class Node:
    name: String
    local: Vec3
    children: Array[Node] = []
    parent: Option[Weak[Node]] = None

    fn init(owns name: String, local: Vec3):
        this.name = name
        this.local = local

    fn add_child(child: Node):
        child.parent = Some(Weak(this))
        this.children.push(child)

    fn world_position() -> Vec3:
        pos = this.local
        current = this.parent
        while current exists as weak:
            if weak.upgrade() exists as p:
                pos = pos + p.local
                current = p.parent
            else:
                break
        return pos

    fn depth() -> int:
        d = 0
        current = this.parent
        while current exists as weak:
            match weak.upgrade():
                Some(p):
                    d += 1
                    current = p.parent
                None:
                    break
        return d

    fn path() -> String:
        if this.parent exists as weak:
            if weak.upgrade() exists as p:
                return f"{p.path()}/{this.name}"
        return this.name

fn print_tree(node: Node):
    indent = "  " * node.depth()
    w = node.world_position()
    l = node.local
    println(f"{indent}{node.name}: local ({l.x}, {l.y}, {l.z}) world ({w.x}, {w.y}, {w.z})")
    for child in node.children:
        print_tree(child)

fn find(node: Node, name: StrView) -> Option[Node]:
    if node.name == name:
        return Some(node)
    for child in node.children:
        if find(child, name) exists as found:
            return Some(found)
    return None

fn count_nodes(node: Node) -> int:
    return 1 + sum(count_nodes(c) for c in node.children)

fn main():
    root = Node("world", Vec3(0, 0, 0))
    house = Node("house", Vec3(10, 0, 5))
    roof = Node("roof", Vec3(0, 3, 0))
    chimney = Node("chimney", Vec3(1, 1, 0))
    tree = Node("tree", Vec3(-4, 0, 2))
    car = Node("car", Vec3(3, 0, -2))
    wheel = Node("wheel", Vec3(0.5, -0.3, 0))
    root.add_child(house)
    house.add_child(roof)
    roof.add_child(chimney)
    root.add_child(tree)
    root.add_child(car)
    car.add_child(wheel)
    print_tree(root)
    println(f"{count_nodes(root)} nodes")

    if find(root, "chimney") exists as n:
        println(f"path of chimney: {n.path()}")

    # move the house: the roof and the chimney move with it
    house.local = house.local + Vec3(0, 0, 10)
    if find(root, "chimney") exists as n:
        w = n.world_position()
        println(f"chimney after the move: ({w.x}, {w.y}, {w.z})")

    if find(root, "garage") is None:
        println("no garage in this scene")
