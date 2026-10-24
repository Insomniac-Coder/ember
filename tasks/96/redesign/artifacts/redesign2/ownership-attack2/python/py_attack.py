# py_attack.py: every Python behaviour the ownership attack report quotes (python -I py_attack.py).
import functools

class Vec2:
    def __init__(self, x, y): self.x = x; self.y = y
class Player:
    def __init__(self, pos, name="ann"): self.pos = pos; self.name = name

# 1. A plain small struct read into a name, then changed through the name.
p = Player(Vec2(1.0, 2.0)); pos = p.pos; pos.x += 1.0
print("copy_struct_name:", p.pos.x)                                   # 2.0: one object
vels = [Vec2(1.0, 1.0), Vec2(2.0, 2.0)]
for v in vels: v.x *= 0.5
print("copy_struct_loop:", [v.x for v in vels])                       # [0.5, 1.0]
a = Player(Vec2(0.0, 0.0)); b = Player(Vec2(0.0, 0.0)); shared = Vec2(1.0, 1.0)
a.pos = shared; b.pos = shared; shared.x = 9.0
print("copy_struct_stored_twice:", a.pos.x, b.pos.x)                 # 9.0 9.0

# 2. A name for a field, then the field replaced through a second handle that may be the same object.
def rename(p, q):
    old = p.name
    q.name = "bob"
    return old
x = Player(Vec2(0, 0), "ann")
print("alias_write_same:", rename(x, x))                              # ann
print("alias_write_other:", rename(x, Player(Vec2(0, 0), "cy")))      # bob (x was renamed by the first call)

# 3. A name for an element, then an element replaced by a position that may be the same one.
class Node:
    def __init__(self, value): self.value = value; self.children = []
xs = [Node(1), Node(2)]
i, j = 0, 0
n = xs[i]; xs[j] = Node(9)
print("element_write_same:", n.value, xs[0].value)                    # 1 9

# 4. A node stored, then popped off again, then used through its old name.
root = Node(0); child = Node(1); child.children.append(Node(3))
root.children.append(child); root.children.pop(); child.children.append(Node(7))
print("store_pop_use:", len(child.children), len(root.children))     # 2 0

# 5. A constructor that calls a helper which sets fields.
class Game:
    def __init__(self, width):
        self.width = width
        self._load()
    def _load(self):
        self.assets = [f"atlas@{self.width}px"]
        self.sounds = ["boom.wav"]
g = Game(640)
print("init_helper_sets_fields:", g.assets, g.sounds)

# 6. A cached value that reads through a second object.
class Owner:
    def __init__(self, name): self.name = name
class Mesh:
    def __init__(self, owner): self.owner = owner
    @functools.cached_property
    def label(self): return self.owner.name + "!"
o = Owner("ann"); m = Mesh(o); first = m.label; o.name = "bob"
print("cached_through_handle:", first, m.label)                       # ann! ann!  (stale)

# 7. dict.get with a list default, and setdefault.
groups = {}
for w in ["ab", "ac", "b"]:
    groups.setdefault(w[0], []).append(w)
print("setdefault:", groups)
print("get_list_default:", groups.get("z", []), len(groups.get("a", [])))

# 8. One object passed as two arguments, one written, the other read.
def bump_then_read(dst, src):
    dst.x = 1.0
    return src.x
s = Vec2(0.0, 0.0)
print("same_arg_twice:", bump_then_read(s, s))                        # 1.0

# 9. A loop over an inherited list field while the same object, typed as the subclass, grows it.
class Script:
    def __init__(self): self.items = [1, 2]
class Door(Script): pass
d = Door(); sv = d; seen = 0
for it in sv.items:
    d.items.append(it)
    seen += 1
    if seen == 6: break
print("inherited_field_grows_in_loop:", seen, len(d.items))           # 6 8 (Python would never end without the break)

# 10. A defaultdict read of a missing key inserts it.
import collections
dd = collections.defaultdict(int); _ = dd["zzz"]
print("defaultdict_read_inserts:", len(dd))                           # 1

# 11. Replacing a list whose elements' drop code walks the same list.
class World:
    def __init__(self): self.items = []
class Item:
    def __init__(self, world): self.world = world; self.hits = 0
    def __del__(self):
        for other in self.world.items: other.hits += 1
        Item.walked += len(self.world.items)
Item.walked = 0
w = World()
for _ in range(3): w.items.append(Item(w))
w.items = []
print("replace_list_drop_walks_it:", Item.walked)                     # 0: Python stores the new list first

# 12. The overlap program of noalias.c.
class Counter:
    def __init__(self): self.n = 5
def reset_then_set(a, b):
    a.n = 0; b.n = 1
    return a.n
c = Counter()
print("reset_then_set_same:", reset_then_set(c, c))                  # 1

# 13. A view of a list's storage kept while the list grows (Python's nearest: memoryview of an array).
import array
v = array.array("i", [5]); s = memoryview(v)
try:
    v.append(7)
    print("view_then_grow: grew")
except BufferError as e:
    print("view_then_grow: BufferError", e)                          # Python refuses the growth while viewed

# ---- added by the second attack ----
# 14. A name for a list inside an object, then the handle variable rebound (name_through_handle.em).
class Holder:
    def __init__(self): self.items = []
h = Holder(); old_h = h; n = h.items; h = Holder(); n.append(1)
print("name_through_handle:", len(n), len(old_h.items), len(h.items))       # 1 1 0: n is the old object's list

# 15. A name for a list field, then a call that MAY replace the field (replaced_in_call.em).
class Bag:
    def __init__(self, dirty): self.items = []; self.dirty = dirty
    def reset(self):
        if self.dirty: self.items = []
for dirty in (False, True):
    b = Bag(dirty); inv = b.items; b.reset(); inv.append(1)
    print(f"replaced_in_call(dirty={dirty}):", len(b.items))               # False: 1 (inv IS b.items); True: 0

# 16. The key expression of `m[k] = m.get(k, 0) + 1` is evaluated twice by Python.
calls = []
def key(): calls.append(1); return "w"
m = {}; m[key()] = m.get(key(), 0) + 1
print("key_expression_calls:", len(calls), m)                              # 2 {'w': 1}

# 17. A value put in a dict, the dict grows, then the value is used through its old name.
class Cell:
    def __init__(self): self.x = 0
d = {}; c1 = Cell(); d["a"] = c1
for k in range(100): d[str(k)] = Cell()                                    # many insertions: the table is rebuilt
c1.x = 5
print("dict_grow_then_name:", d["a"].x)                                    # 5
# ... and the key removed, then the old name used.
d2 = {}; c2 = Cell(); d2["a"] = c2; d2.pop("a"); c2.x = 7
print("dict_pop_then_name:", c2.x, len(d2))                                # 7 0

# 18. A generator walks a list; between two next() calls the caller appends to it (gen_push_between.em).
def walk(xs):
    for x in xs:
        yield x
items = [1, 2]; g = walk(items); first = next(g); items.append(99)
print("gen_push_between:", first, list(g))                                 # 1 [2, 99]
# ... or stores it into an object and appends there (gen_store_between.em).
class World2:
    def __init__(self): self.items = []
items = [1, 2]; w2 = World2(); g = walk(items); first = next(g); w2.items = items; w2.items.append(99)
print("gen_store_between:", first, list(g))                                # 1 [2, 99]
