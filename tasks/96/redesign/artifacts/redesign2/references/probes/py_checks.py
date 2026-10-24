# py_checks.py: the Python programs of phase3b-references.md, with what Python does. Run: python -I py_checks.py
from math import pi

# A5: a conditionally chosen part, and a component reached through a dict of pools keyed by class.
class Pos:
    def __init__(self, x): self.x = x
class Pool:
    def __init__(self): self.dense, self.index = [], {}
class Registry:
    def __init__(self): self.pools = {}
    def get(self, T, e):
        pool = self.pools[T]
        return pool.dense[pool.index[e]]
reg = Registry(); reg.pools[Pos] = Pool(); reg.pools[Pos].dense.append(Pos(1.0)); reg.pools[Pos].index[7] = 0
reg.get(Pos, 7).x += 1.0
assert reg.pools[Pos].dense[0].x == 2.0

# A5: a property with a setter: `a.degrees += 90` reads, changes and writes back.
class Angle:
    def __init__(self, r): self.radians = r
    @property
    def degrees(self): return self.radians * 180.0 / pi
    @degrees.setter
    def degrees(self, d): self.radians = d * pi / 180.0
a = Angle(0.0); a.degrees += 90.0
assert abs(a.radians - pi / 2) < 1e-12

# A6: handles kept in a list; an object holding a lexer over a text.
class Registry2:
    def __init__(self): self.hp = [10, 20]
class View:
    def __init__(self, reg, entity): self.reg, self.entity = reg, entity
    def hp(self): return self.reg.hp[self.entity]
r2 = Registry2(); vs = [View(r2, 0), View(r2, 1)]
assert sum(v.hp() for v in vs) == 30

# A7: a document that owns its text and keeps its words.
class Document:
    def __init__(self, text):
        self.text = text
        self.words = text.split()
d = Document("the quick fox")
assert d.words == ["the", "quick", "fox"]

# A22: a view of a temporary; the best word kept after the loop; a word returned from a temporary.
def first_word_len(line): return len(line.lower().strip())
assert first_word_len(" Hello ") == 5
def longest(lines):
    best = ""
    for line in lines:
        w = line.lower().strip()
        if len(w) > len(best): best = w
    return len(best)
assert longest(["a ", " bb"]) == 2
def first_word(line): return line.lower().strip()
assert first_word(" Hello ") == "hello"

# A23: the state machine; and keeping a field's old value across an overwrite.
class Walking:
    def __init__(self, path): self.path = path
class Running:
    def __init__(self, path): self.path = path
class Idle: pass
class Npc:
    def __init__(self, path): self.state = Walking(path)
    def update(self, hurry):
        if isinstance(self.state, Walking):
            self.state.path.pop()
            if not self.state.path: self.state = Idle()
            elif hurry: self.state = Running(self.state.path)
n = Npc([1, 2, 3]); n.update(True)
assert isinstance(n.state, Running) and n.state.path == [1, 2]
class Person:
    def __init__(self, name): self.name, self.log = name, []
def rename(p, new_name):
    old = p.name
    p.name = new_name
    p.log.append(f"renamed {old}")
p = Person("ann"); rename(p, "bob")
assert p.log == ["renamed ann"] and p.name == "bob"
print("all Python checks passed")
