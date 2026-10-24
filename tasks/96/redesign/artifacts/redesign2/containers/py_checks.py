# Python behaviour the containers report relies on (run: C:/Python314/python.exe -I py_checks.py).

class Enemy:
    def __init__(self, hp, splits=False):
        self.hp, self.splits = hp, splits
    def __repr__(self):
        return f"E{self.hp}"

# A4: appending during a for loop: the appended item is visited (the loop runs to the live length).
xs = [1, 2]
seen = []
for x in xs:
    seen.append(x)
    if x == 1:
        xs.append(3)
print("A4 append visited:", seen)

# A4: removing the current item skips the next one.
es = [Enemy(0), Enemy(0), Enemy(5)]
for e in es:
    if e.hp <= 0:
        es.remove(e)
print("A4 remove-current leaves:", es)

# A4: the game loop of phase 1 (spawn, then despawn the current enemy).
es = [Enemy(0, True), Enemy(0, False), Enemy(3)]
visited = []
for e in es:
    visited.append(repr(e))
    if e.hp <= 0:
        if e.splits:
            es.append(Enemy(5))
        es.remove(e)
print("A4 game loop visited:", visited, "left:", es)

# A4: a method that removes from the list being walked (A1 + A4).
class World:
    def __init__(self):
        self.enemies = [Enemy(0), Enemy(0), Enemy(4)]
        self.score = 0
    def kill(self, e):
        self.enemies.remove(e)
        self.score += 10
    def update(self):
        for e in self.enemies:
            if e.hp <= 0:
                self.kill(e)
w = World(); w.update()
print("A4 via method left:", w.enemies, "score:", w.score)

# A4: list.extend(itself) doubles (the library copies first).
ys = [1, 2]; ys.extend(ys); print("extend self:", ys)

# A2: a name for an element keeps the object through append and through removal.
enemies = [Enemy(100)]
boss = enemies[0]
enemies.append(Enemy(boss.hp // 2))
boss.hp //= 2
print("A2 after append:", enemies)
picked = enemies[0]
enemies.remove(picked)
print("A2 removed but still usable:", picked, "list:", enemies)

# A3: the same body passed twice works (one object, two names).
class Body:
    def __init__(self, vx): self.vx = vx
def collide(a, b):
    a.vx, b.vx = b.vx, a.vx
bs = [Body(1.0), Body(-1.0)]
collide(bs[0], bs[0])
print("A3 same body twice:", bs[0].vx)

# A25: a list of words outlives the lowered temporary; a function may return it.
def words_of(line):
    return line.lower().split()
print("A25 returned words:", words_of("A b C"))

# ECS in plain Python (one dict per component, entity id -> component): deleting an entity while
# walking a dict raises; the Python habit is to walk a copy of the keys.
pos = {i: [float(i), 0.0] for i in range(6)}
vel = {i: [1.0, 0.5] for i in range(0, 6, 2)}
next_id = 6
try:
    for e in vel:
        if e == 2:
            del pos[e]; del vel[e]
            pos[next_id] = [0.0, 0.0]; vel[next_id] = [1.0, 1.0]; next_id += 1
except RuntimeError as err:
    print("ECS dict walk:", err)
for e in list(vel):
    p, v = pos[e], vel[e]
    p[0] += v[0]
    if e == 4:
        del pos[e]; del vel[e]
        pos[next_id] = [0.0, 0.0]; vel[next_id] = [1.0, 1.0]; next_id += 1
print("ECS walk a copy:", sorted(vel))
