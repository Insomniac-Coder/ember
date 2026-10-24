# W2 in the combined model's proposed syntax (phase3b-combined.md 6.2). Checked with today's
# `ember check` only to record what today's compiler says; it is not expected to compile today.
state: u64 = 42

fn rand() -> int:
    global state
    state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407)
    return (state >> 33) as int

class Node:
    value: i32
    parent: Option[Node] = backlink(children)
    children: Array[Node] = []

fn main():
    n_nodes = 1_000_000
    nodes = [Node(value=0)]
    for i in 1..n_nodes:
        n = Node(value=(i % 1000) as i32)
        nodes.push(n)
        nodes[rand() % i].children.push(n)
    depth_sum = 0
    for p in 0..10:
        for n in nodes:
            m = n.parent
            while m is not None:
                depth_sum += 1
                m = m.parent
        for i in 1..n_nodes:
            if (i + p) % 100 == 0:
                nodes[rand() % i].children.push(nodes[i])
    checksum = depth_sum
    for n in nodes:
        checksum += n.value as int * len(n.children)
    println(checksum)
