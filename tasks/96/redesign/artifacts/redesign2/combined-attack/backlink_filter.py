# backlink_filter.py: the Python twin of backlink_filter.c (problem 6 of phase3b-combined-attack.md).
class Node:
    def __init__(self, value): self.value = value; self.parent = None; self.children = []
def reparent(c, new):
    c.parent.children.remove(c); c.parent = new; new.children.append(c)
old, new = Node(0), Node(1)
for v in [3, 7, 9, 2]:
    n = Node(v); n.parent = old; old.children.append(n)
for c in old.children:
    if c.value > 5:
        reparent(c, new)
print("python, loop over the list itself:", [c.value for c in new.children], [c.value for c in old.children])
old, new = Node(0), Node(1)
for v in [3, 7, 9, 2]:
    n = Node(v); n.parent = old; old.children.append(n)
for c in list(old.children):
    if c.value > 5:
        reparent(c, new)
print("python, loop over a copy:", [c.value for c in new.children], [c.value for c in old.children])
