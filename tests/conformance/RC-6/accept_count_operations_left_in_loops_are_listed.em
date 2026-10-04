#$ test: run-pass
#$ rules: RC-6, RC-1, RC-2d
#$ profiles: debug, release, shipping
#$ stdout: 9 3 2
# `[RC-6]` — `ember inspect --counts` lists the retains and releases left
# inside loops, with the reason each could not be removed (the milestone
# `inspect_counts_lists_count_operations_left_in_loops` reads this program's
# report). Pushing `node` into a list and storing `other` into a field keep
# counts of their own, and the store releases the field's old value; the loop
# over the list's borrowed handles has none.

class Node:
    value: int

class Holder:
    item: Node

fn main():
    node = Node(1)
    other = Node(2)
    holder = Holder(Node(0))
    nodes: Array[Node] = Array[Node]()
    total = 0
    for i in 0..3:
        nodes.push(node)
        holder.item = other
        total += holder.item.value
    for item in nodes:
        total += item.value
    println(total, nodes.len(), holder.item.value)
