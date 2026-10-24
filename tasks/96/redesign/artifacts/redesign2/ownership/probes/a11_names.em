# A11: the Python lines that rely on names, each in its own function.
struct Node:
    value: int
    children: Array[Node] = []

struct Player:
    name: String
    hp: int

fn keep(owned p: Player):
    pass

# 1. Two names for one list; a change through one is seen through the other.
fn alias_then_push():
    items: Array[int] = [1, 2]
    snapshot = items
    items.push(3)
    println(snapshot)

# 2. Build a node, store it, keep using its name (the commonest Python shape).
fn store_then_use():
    root = Node(value=0)
    child = Node(value=1)
    root.children.push(child)
    child.value = 5
    println(root.children[0].value)

# 3. A list of rows filled after they are stored.
fn rows():
    matrix: Array[Array[int]] = []
    row: Array[int] = []
    matrix.push(row)
    row.push(1)
    println(matrix)

# 4. Read a field into a name, then give the whole value away, then use the name.
fn field_then_give(owned p: Player):
    n = p.name
    keep(p)
    println(n)

# 5. Rebinding one name leaves the other with the old value (Python's meaning).
fn rebind():
    a: Array[int] = [1]
    b = a
    a = [9]
    println(b)

fn main():
    alias_then_push()
    store_then_use()
    rows()
    rebind()
