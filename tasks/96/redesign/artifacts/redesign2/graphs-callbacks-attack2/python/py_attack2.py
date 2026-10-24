# py_attack2.py: what Python does with each program the attack report uses. Run: python -I py_attack2.py
import gc
import weakref
from typing import Generic, TypeVar

results = {}

# 1. isinstance narrowing of a local that a closure writes (attacks2.exe 1/2)
class Shape: pass
class Circle(Shape):
    def __init__(self): self.r = 1.5
class Square(Shape): pass
def narrowed_local_written_by_closure():
    s = Circle()
    def reset():
        nonlocal s
        s = Square()
    hs = [reset]
    if isinstance(s, Circle):
        hs[0]()
        try:
            return s.r
        except AttributeError as e:
            return f"AttributeError: {e}"
results["narrowed_local_written_by_closure"] = narrowed_local_written_by_closure()

# 2. a stored closure called with an argument whose evaluation removes it (attacks2.exe 3/4)
class Bus:
    def __init__(self): self.handlers = []
    def clear_and_count(self):
        n = len(self.handlers); self.handlers.clear(); return n
def callee_then_args():
    out = []
    bus = Bus(); name = "first"
    bus.handlers.append(lambda n: out.append(f"{name} {n}"))
    bus.handlers[0](bus.clear_and_count())
    return out
results["callee_evaluated_before_args"] = callee_then_args()

# 3. a closure lending its captured list to a call that drops the closure (attacks2.exe 5/6)
class Bus2:
    def __init__(self): self.handlers = []
    def report(self, xs):
        self.handlers.clear()
        return xs[0]
def lent_state():
    bus = Bus2()
    seen = [1, 2, 3]
    got = []
    bus.handlers.append(lambda b: got.append(b.report(seen)))
    bus.handlers[0](bus)
    return got
results["closure_lends_its_state"] = lent_state()

# 4. assigning a link the value it already holds (attacks2.exe 7/8)
class LNode:
    def __init__(self, v): self.v = v; self.next = None; self.prev = None
def self_assign():
    a, c = LNode(1), LNode(2)
    a.next = c; c.prev = a
    a.next = a.next
    return a.next.v
results["link_self_assign"] = self_assign()

# 4b. Python's two-line unlink from a doubly linked list
def unlink_two_lines():
    a, b, c = LNode(1), LNode(2), LNode(3)
    a.next = b; b.prev = a; b.next = c; c.prev = b
    node = b
    node.prev.next = node.next
    node.next.prev = node.prev
    out, n = [], a
    while n: out.append(n.v); n = n.next
    return out, c.prev.v
results["unlink_two_lines"] = unlink_two_lines()

# 5. a weak list after a removal, growth and a death (attacks2.exe 9/10)
class Observer:
    def __init__(self, i): self.id = i
def weak_list():
    o = {i: Observer(i) for i in range(1, 7)}
    observers = [weakref.ref(o[1]), weakref.ref(o[2]), weakref.ref(o[3])]
    del observers[0]
    observers += [weakref.ref(o[4]), weakref.ref(o[5]), weakref.ref(o[6])]
    del o[2]; gc.collect()
    return [None if r() is None else r().id for r in observers]
results["weak_list_after_shift_and_death"] = weak_list()

# 6. a list emptied by a closure while a loop walks it (attacks2.exe 11/12)
def loop_over_captured():
    xs = ["ab", "cde", "f"]
    def clear(): xs.clear()
    hs = [clear]
    total = 0
    for x in xs:
        hs[0]()
        total += len(x)
    return total
results["list_cleared_by_closure_during_loop"] = loop_over_captured()

# 7. duck typing in a generic helper (probes/a16_duck_method.em)
class Crate:
    def __init__(self): self.kg = 5
    def weight(self): return self.kg
class Sack:
    def __init__(self): self.kg = 2
    def weight(self): return self.kg
def heaviest(xs):
    best = 0
    for x in xs:
        w = x.weight()
        if w > best: best = w
    return best
results["duck_typed_method"] = [heaviest([Crate(), Crate()]), heaviest([Sack()])]

# 8. the filter idiom on a tree's child list
class Node:
    def __init__(self, v): self.v = v; self.parent = None; self.children = []
def filter_children():
    p = Node(0)
    for i in range(1, 7):
        c = Node(i); c.parent = p; p.children.append(c)
    old = p.children
    p.children = [c for c in p.children if c.v % 2 == 0]
    dropped = [c.v for c in old if c not in p.children]
    return [c.v for c in p.children], dropped, [c.parent is p for c in old]
results["filter_idiom_children"] = filter_children()

# 9. isinstance with a generic class
T = TypeVar("T")
class Holder(Generic[T]):
    def __init__(self, value): self.value = value
def generic_isinstance():
    xs = [Holder("seven"), Holder(7)]
    plain = [isinstance(x, Holder) for x in xs]
    try:
        isinstance(xs[1], Holder[int])
        param = "accepted"
    except TypeError as e:
        param = f"TypeError: {e}"
    return plain, param
results["isinstance_generic"] = generic_isinstance()

# 10. a closure that writes a captured variable inside a hot loop (bench/boxloop.c)
def boxed_in_loop():
    total = 0
    def read(): return total
    callbacks = [read]
    for x in range(1000):
        total += x
    return callbacks[0]()
results["boxed_variable_hot_loop"] = boxed_in_loop()

for k, v in results.items():
    print(f"{k}: {v}")
