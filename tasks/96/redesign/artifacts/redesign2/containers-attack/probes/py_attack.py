# Python behaviour the containers attack relies on. Run: C:/Python314/python.exe -I py_attack.py
from array import array

class Enemy:
    def __init__(self, hp, name="e"):
        self.hp, self.name = hp, name
    def __repr__(self):
        return f"E{self.hp}"

# A2 attack 1: the swap idiom through a temporary name.
xs = [Enemy(1), Enemy(2)]
tmp = xs[0]; xs[0] = xs[1]; xs[1] = tmp
print("A2 swap idiom:", xs)
assert repr(xs) == "[E2, E1]"

# A2 attack 1b: a name keeps the old object when its slot is replaced.
xs = [Enemy(1)]
old = xs[0]; xs[0] = Enemy(9)
print("A2 slot replaced, old name:", old, "list:", xs)
assert old.hp == 1 and xs[0].hp == 9

# A2 attack 2: an element passed to a method that removes other elements of its list.
class World:
    def __init__(self):
        self.enemies = [Enemy(0), Enemy(50), Enemy(70)]
    def purge(self):
        self.enemies[:] = [x for x in self.enemies if x.hp > 0]
    def hit(self, e):
        e.hp -= 10
        self.purge()
w = World(); w.hit(w.enemies[1])
print("A2 hit + purge:", w.enemies)
assert repr(w.enemies) == "[E40, E70]"

# A2 attack 3: a name held across a call (through a handler object) that removes from the list.
class Handler:
    def on_tick(self, world):
        world.purge()
w = World(); boss = w.enemies[2]; Handler().on_tick(w); boss.hp -= 1
print("A2 name across a removing callback:", w.enemies, "boss:", boss)
assert repr(w.enemies) == "[E50, E69]"

# A2 attack 4: two names for one element, the element removed: both keep the one object.
xs = [Enemy(5)]
a = xs[0]; b = xs[0]; xs.pop(0); a.hp = 7
print("A2 two names, element removed:", a, b)
assert b.hp == 7

# A3: Python list slices are copies, so a function given two slices changes neither.
def add(d, s):
    for k in range(len(d)):
        d[k] = d[k] * 0.5 + s[k]
xs = [float(k) for k in range(8)]
add(xs[0:4], xs[2:6])
print("A3 list slices are copies:", xs[:4])
assert xs[:4] == [0.0, 1.0, 2.0, 3.0]
# A3: memoryview slices are views and overlap element by element (C's meaning).
m = array('d', [float(k) for k in range(8)])
add(memoryview(m)[2:6], memoryview(m)[0:4])
print("A3 memoryview overlap (dst = src shifted by 2):", list(m[:6]))
# As if no overlap the result would be [0,1, 2*0.5+0, 3*0.5+1, 4*0.5+2, 5*0.5+3] = [0,1,1,2.5,4,5.5].
# With C's element-by-element meaning dst[2] reads the already-updated m[2]: m[4] = 4*0.5 + 1.0 = 3, m[5] = 5*0.5 + 2.5 = 5.
assert list(m[:6]) == [0.0, 1.0, 1.0, 2.5, 3.0, 5.0]

# A3: extending a list with a slice of itself (the slice is a copy taken first).
ys = [1, 2, 3]; ys.extend(ys[0:2])
print("A3 extend with own slice:", ys)
assert ys == [1, 2, 3, 1, 2]

# A4: code called from inside the loop sees a removal at once.
lens = []
class W2:
    def __init__(self): self.enemies = [Enemy(0), Enemy(5), Enemy(0), Enemy(5)]
    def on_step(self): lens.append(len(self.enemies))
w2 = W2()
for e in list(w2.enemies):
    if e.hp <= 0: w2.enemies.remove(e)
    w2.on_step()
print("A4 len seen by a callback after each step:", lens)
assert lens == [3, 3, 2, 2]

# A4: a polymorphic update method that removes its own enemy from the list being walked.
class Grunt:
    def __init__(self, hp): self.hp = hp
    def __repr__(self): return f"E{self.hp}"
    def update(self, world):
        self.hp -= 1
        if self.hp <= 0:
            world.enemies.remove(self)
class W3:
    def __init__(self): self.enemies = [Grunt(1), Grunt(1), Grunt(3)]
    def step(self):
        for e in self.enemies:
            e.update(self)
w3 = W3(); w3.step()
print("A4 update removes itself (Python skips the next one):", w3.enemies)
assert repr(w3.enemies) == "[E1, E2]"

# A25: the word list built line by line (the usual file loop).
text = "The cat\nSAT on\nthe Mat"
all_words = []
for line in text.split("\n"):
    all_words.extend(line.lower().split())
print("A25 words from every line:", all_words)
assert all_words == ["the", "cat", "sat", "on", "the", "mat"]

print("all Python attack checks passed")
