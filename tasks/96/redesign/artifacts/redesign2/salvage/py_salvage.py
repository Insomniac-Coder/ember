# Python behaviours the salvage report quotes. Run: python -I py_salvage.py
from dataclasses import dataclass, field

# 1. lists are shared, not copied
matrix = []; row = []
matrix.append(row); row.append(1)
print("1a matrix:", matrix)
d = {}
lst = d.get("a")
if lst is None:
    lst = []
    d["a"] = lst
lst.append(1); lst.append(2)
print("1b buckets:", d)

# 2. a value read out of a field survives the field being overwritten, even by a callee
class Person:
    def __init__(self, name): self.name = name
def rename(p, new): p.name = new
p = Person("Ann"); old = p.name; rename(p, "Bob")
print("2a old name:", old)
class Worker:
    def __init__(self): self.pending = ["j1", "j2"]
    def reset(self): self.pending = []
w = Worker(); pending = w.pending; w.reset()
print("2b drained:", [j for j in pending])

# 3. parent pointers: clearing the children list does not destroy the parent
class Node:
    def __init__(self): self.parent = None; self.children = []
root = Node(); c = Node()
root.children.append(c); c.parent = root
root.children.clear()
print("3 root children after clear:", len(root.children), "c.parent is root:", c.parent is root)

# 4. a handler unregisters itself from inside its own call
total = 0
class Dispatcher:
    def __init__(self): self.handlers = []; self.current = 0
def make_handler(m):
    counter = 0
    def h(dp, k, v):
        nonlocal counter
        global total
        counter += v * m; total += 1
        if k % 3 == 2:
            dp.handlers.pop(dp.current)
            dp.handlers.append(make_handler(m + 1))
        return counter
    return h
dp = Dispatcher(); dp.handlers = [make_handler(1), make_handler(2)]
for k in range(9):
    dp.current = k % len(dp.handlers); dp.handlers[dp.current](dp, k, k)
print("4 total:", total, "handlers:", len(dp.handlers))

# 5. a closure that clears the list holding it, then keeps using its own state
hs = []
def make():
    n = 0
    def f():
        nonlocal n
        hs.clear(); n += 1
        return n
    return f
hs.append(make())
print("5 closure after clearing its list:", hs[0]())

# 6. a generator over a local list, returned from the function that made the list
def evens(xs):
    for x in xs:
        if x % 2 == 0: yield x
def mk():
    local = [2, 4, 6]
    return evens(local)
print("6 generator over a dead frame's list:", next(mk()))

# 7. writing to an object after it was removed from its list
class Ship:
    def __init__(self): self.hp = 5
class World:
    def __init__(self): self.ships = [Ship()]
    def kill(self, s): self.ships.remove(s)
def hit(wd, s):
    s.hp -= 10
    if s.hp <= 0: wd.kill(s)
    s.hp = 0
wd = World(); s0 = wd.ships[0]; hit(wd, s0)
print("7 removed ship hp:", s0.hp, "ships left:", len(wd.ships))

# 8. an alias to an element that is popped and its holder cleared
@dataclass
class Item:
    name: str
class Bag:
    def __init__(self): self.items = [Item("sword")]; self.trash = []
bag = Bag(); it = bag.items[0]
bag.trash.append(bag.items.pop()); bag.trash.clear()
print("8 alias after pop and clear:", it.name)

# 9. assigning to a loop variable does not write into the list
lines = ["  a ", "b "]
for line in lines:
    line = line.strip()
print("9 lines unchanged:", lines)

# 10. a match arm that replaces the state it matched keeps the old payload
@dataclass
class Walking:
    path: list
class Idle: pass
state = Walking([1, 2, 3]); done = []
match state:
    case Walking(path=path):
        state = Idle()
        for step in path: done.append(step)
print("10 old payload after replacing the state:", done)

# 11. a struct-like value appended to two lists and one list cleared
m = Item("mesh"); a = [m]; b = [m]; a.clear()
print("11 b still has it:", b[0].name)

# 12. a function changes an element while a closure it calls replaces the list holding that element
class Squad:
    def __init__(self, units): self.units = units
def squads_demo():
    squads = [Squad([1, 2])]
    def grow(s, f):
        f()
        s.units.append(3)
        return s.units
    def replace():
        nonlocal squads
        squads = []
    return grow(squads[0], replace), squads
print("12 element changed after its list was replaced:", squads_demo())
