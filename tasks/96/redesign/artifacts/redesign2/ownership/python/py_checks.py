# py_checks.py: every Python behaviour the ownership report quotes, checked (python -I py_checks.py).
import functools, collections

class Node:
    def __init__(self, value): self.value = value; self.children = []

# A11: store a node, keep using its name.
root = Node(0); child = Node(1); root.children.append(child); child.value = 5
print("store_then_use:", root.children[0].value)                      # 5

# A11: a second name for one list.
items = [1, 2]; snapshot = items; items.append(3)
print("alias_then_push:", snapshot)                                   # [1, 2, 3]

# A11: rows filled after they are stored.
matrix = []; row = []; matrix.append(row); row.append(1)
print("rows:", matrix)                                                 # [[1]]

# A11: rebinding one name leaves the other with the old value.
a = [1]; b = a; a = [9]
print("rebind:", b)                                                    # [1]

# A11: a field read into a name keeps the old value when the field is replaced.
class P:
    def __init__(self): self.weapon = "bow"
    def upgrade(self): self.weapon = "axe"
p = P(); w = p.weapon; p.upgrade()
print("name_keeps_old_after_call:", w)                                 # bow

# A11: the same object appended n times gives n names for one object.
t = Node(0); v = [t for _ in range(3)]; v[0].value = 7
print("append_same_n_times:", [x.value for x in v])                   # [7, 7, 7]

# A31: functools.cached_property computes once.
calls = 0
class Mesh:
    def __init__(self, verts): self.verts = verts
    @functools.cached_property
    def radius(self):
        global calls; calls += 1
        return max(abs(x) for x in self.verts)
m = Mesh([1.0, -2.0]); m.radius; m.radius
print("cached_property:", m.radius, "computed", calls, "time(s)")      # 2.0 computed 1 time(s)
m.verts.append(-5.0)
print("cached_property after verts changed:", m.radius)                # still 2.0: Python does not invalidate

# A21: the spellings of a word count.
counts = {}
for wd in "a b a".split(): counts[wd] = counts.get(wd, 0) + 1
dd = collections.defaultdict(int)
for wd in "a b a".split(): dd[wd] += 1
print("get_default:", counts, "defaultdict:", dict(dd))                # {'a': 2, 'b': 1} twice
try:
    c2 = {}; c2["a"] += 1
except KeyError as e:
    print("missing_key_plus_equals: KeyError", e)

# A9: transfer between two objects, and the same object twice.
class Bag:
    def __init__(self, items): self.items = items
x = Bag(["k"]); y = Bag([])
for it in x.items: y.items.append(it)
print("transfer:", y.items)                                            # ['k']
z = Bag(["k"]); n = 0
for it in z.items:
    z.items.append(it); n += 1
    if n == 5: break
print("same_object_grows_while_iterating: stopped by hand after", n, "steps; len", len(z.items))

# A18: reading a field that __init__ has not set yet.
class Game:
    def __init__(self):
        self.assets = self.load_assets()      # reads self.renderer before it is set
        self.renderer = 640
    def load_assets(self): return [self.renderer]
try:
    Game()
except AttributeError as e:
    print("read_before_set: AttributeError", e)
