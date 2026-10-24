# Every Python behaviour the graphs-and-callbacks report quotes, asserted. Run: python -I py_checks.py
import gc, weakref
from functools import singledispatch
from xml.dom.minidom import Document

out = []
def show(name, value):
    out.append(f"{name}: {value!r}")

# A8 -------------------------------------------------------------------------------------------
class Node:
    def __init__(self, name):
        self.name = name; self.parent = None; self.children = []
    def add(self, child):
        child.parent = self
        self.children.append(child)

def path(n):
    return n.name if n.parent is None else path(n.parent) + "/" + n.name

root = Node("root"); leaf = Node("leaf"); root.add(leaf)
assert path(leaf) == "root/leaf"; show("tree_path", path(leaf))

# the root's last name goes, a leaf is still held: Python keeps the root alive through leaf.parent
def keep_leaf():
    r = Node("r"); l = Node("l"); r.add(l)
    return l
l = keep_leaf()
assert l.parent is not None and l.parent.name == "r"; show("root_kept_alive_by_child_parent", l.parent.name)

# re-parenting in Python is three lines; forgetting the first leaves the node in two child lists
a, b, c = Node("a"), Node("b"), Node("c")
a.add(c)
b.children.append(c); c.parent = b          # forgot a.children.remove(c)
assert c in a.children and c in b.children; show("reparent_forgot_remove_in_both", [len(a.children), len(b.children)])

# the DOM (stdlib minidom): appendChild of a node that has a parent moves it
doc = Document(); p1 = doc.createElement("p1"); p2 = doc.createElement("p2"); x = doc.createElement("x")
p1.appendChild(x); p2.appendChild(x)
assert len(p1.childNodes) == 0 and x.parentNode is p2; show("dom_appendChild_moves", [len(p1.childNodes), x.parentNode.tagName])

# a cycle of strong references is freed by the cycle collector
class City:
    def __init__(self, name): self.name = name; self.roads = []
freed = []
def make_cycle():
    x = City("a"); y = City("b"); x.roads.append(y); y.roads.append(x)
    weakref.finalize(x, freed.append, "a"); weakref.finalize(y, freed.append, "b")
make_cycle()
assert freed == []                            # counting alone does not free it
gc.collect()
assert sorted(freed) == ["a", "b"]; show("cycle_freed_by_gc", sorted(freed))

# A12 ------------------------------------------------------------------------------------------
class Button:
    def __init__(self): self.handlers = []
    def on_click(self, f): self.handlers.append(f)
    def click(self):
        for h in self.handlers: h()

def game():
    score = 0
    def on_click():
        nonlocal score
        score += 1
    b = Button(); b.on_click(on_click); b.click()
    return score
assert game() == 1; show("button_score", game())

def make_counter():
    count = 0
    def inc():
        nonlocal count
        count += 1
        return count
    return inc
cnt = make_counter(); cnt()
assert cnt() == 2; show("counter_factory", 2)

def make_pair():
    count = 0
    def inc():
        nonlocal count
        count += 1
    def get(): return count
    return inc, get
inc, get = make_pair(); inc(); inc()
assert get() == 2; show("two_closures_share", get())

def make_read_later():
    n = 1
    f = lambda: n
    n = 2
    return f
assert make_read_later()() == 2; show("read_after_capture", make_read_later()())

fs = [lambda: i for i in range(3)]
assert [f() for f in fs] == [2, 2, 2]; show("loop_capture_late_binding", [f() for f in fs])

# A32 ------------------------------------------------------------------------------------------
class Ui:
    def __init__(self): self.handlers = []; self.clicks = 0
    def fire(self):
        for h in self.handlers: h(self)
def count(ui): ui.clicks += 1
added = []
def add_more(ui):
    if not added:
        added.append(1); ui.handlers.append(count)
ui = Ui(); ui.handlers += [count, add_more]; ui.fire()
assert ui.clicks == 2; show("handler_added_during_dispatch_runs_same_dispatch", ui.clicks)

calls = []
class Bus:
    def __init__(self): self.handlers = []
    def emit(self, v):
        for h in self.handlers: h(self, v)
def once(bus, v):
    calls.append(("once", v)); bus.handlers.remove(once)
def always(bus, v):
    calls.append(("always", v))
bus = Bus(); bus.handlers += [once, always]; bus.emit(5); bus.emit(7)
assert calls == [("once", 5), ("always", 7)]; show("remove_self_skips_next_in_same_dispatch", calls)

# a handler that re-enters the dispatch: Python allows the recursion
depth = []
class Re:
    def __init__(self): self.handlers = []
    def fire(self, n):
        for h in list(self.handlers): h(self, n)
def rec(r, n):
    depth.append(n)
    if n < 3: r.fire(n + 1)
r = Re(); r.handlers.append(rec); r.fire(0)
assert depth == [0, 1, 2, 3]; show("reentrant_handler", depth)

# A13 ------------------------------------------------------------------------------------------
class Shape: pass
class Circle(Shape):
    def __init__(self, r): self.r = r
class Square(Shape):
    def __init__(self, s): self.s = s
shapes = [Circle(1.0), Square(2.0)]
assert sum(s.r for s in shapes if isinstance(s, Circle)) == 1.0; show("isinstance_filter", 1.0)

class Pool:
    def __init__(self): self.dense = []
pools = {int: Pool(), float: Pool()}
pools[int].dense.append(10)
assert pools[int].dense[0] == 10; show("dict_keyed_by_type", pools[int].dense)

# A16 ------------------------------------------------------------------------------------------
def sum_all(xs):
    acc = xs[0]
    for x in xs[1:]: acc = acc + x
    return acc
assert sum_all([1, 2, 3]) == 6 and sum_all([1.5, 2.5]) == 4.0; show("duck_sum", [sum_all([1, 2, 3]), sum_all([1.5, 2.5])])
try:
    sum_all([object(), object()]); show("duck_sum_error", "none")
except TypeError as e:
    show("duck_sum_error_at_run_time", str(e))

@singledispatch
def describe(x): return repr(x)
@describe.register
def _(x: int): return f"int {x}"
assert describe(3) == "int 3" and describe("a") == "'a'" and describe(True) == "int True"
show("singledispatch_most_specific", [describe(3), describe("a"), describe(True)])

print("\n".join(out))
print("all Python checks passed")
