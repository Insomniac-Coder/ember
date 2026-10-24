# W2 (tree with parent pointers) written in Python, the twin of the Ember program in
# phase3b-combined.md section 6.2. Must print the known checksum 647034814. Run: python -I w2.py
N = 1_000_000
state = 42


def rand():                                  # harness.h's h_rand()
    global state
    state = (state * 6364136223846793005 + 1442695040888963407) & 0xFFFF_FFFF_FFFF_FFFF
    return state >> 33


class Node:
    __slots__ = ("value", "parent", "children")
    def __init__(self, value):
        self.value = value
        self.parent = None
        self.children = []


def main():
    nodes = [Node(0)]
    for i in range(1, N):
        n = Node(i % 1000)
        nodes.append(n)
        p = nodes[rand() % i]
        n.parent = p
        p.children.append(n)
    depth_sum = 0
    for p in range(10):
        for n in nodes:
            m = n.parent
            while m is not None:
                depth_sum += 1
                m = m.parent
        for i in range(1, N):
            if (i + p) % 100 == 0:
                node = nodes[i]
                new_parent = nodes[rand() % i]
                node.parent.children.remove(node)
                node.parent = new_parent
                new_parent.children.append(node)
    print(depth_sum + sum(n.value * len(n.children) for n in nodes))


main()
