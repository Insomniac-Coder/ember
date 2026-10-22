# A scene graph: nodes with local offsets, children and weak parent links, printing each node's world position.

from std.math import Vec3

class Node:
    name: String
    local: Vec3
    children: Array[Node] = []
    parent: Option[Weak[Node]] = None

    fn init(self, owned name: String, local: Vec3):
        self.name = name
        self.local = local

    fn add_child(self, child: Node):
        child.parent = Some(Weak(self))
        self.children.push(child)

    fn world_position(self) -> Vec3:
        pos = self.local
        current = self.parent
        while Some(weak) = current:
            if Some(p) = weak.upgrade():
                pos = pos + p.local
                current = p.parent
            else:
                break
        return pos

    fn depth(self) -> int:
        d = 0
        current = self.parent
        while Some(weak) = current:
            match weak.upgrade():
                Some(p):
                    d += 1
                    current = p.parent
                None:
                    break
        return d

    fn path(self) -> String:
        if Some(weak) = self.parent:
            if Some(p) = weak.upgrade():
                return f"{p.path()}/{self.name}"
        return self.name.clone()

fn print_tree(node: Node):
    indent = "  ".repeat(node.depth())
    w = node.world_position()
    l = node.local
    println(f"{indent}{node.name}: local ({l.x}, {l.y}, {l.z}) world ({w.x}, {w.y}, {w.z})")
    for child in node.children:
        print_tree(child)

fn find(node: Node, name: str) -> Option[Node]:
    if node.name == name:
        return Some(node)
    for child in node.children:
        if Some(found) = find(child, name):
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

    if Some(n) = find(root, "chimney"):
        println(f"path of chimney: {n.path()}")

    # move the house: the roof and the chimney move with it
    house.local = house.local + Vec3(0, 0, 10)
    if Some(n) = find(root, "chimney"):
        w = n.world_position()
        println(f"chimney after the move: ({w.x}, {w.y}, {w.z})")

    if find(root, "garage") is None:
        println("no garage in this scene")
