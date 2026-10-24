# W3 (game objects referring to each other) written in Python, the twin of the Ember program in
# phase3b-combined.md section 6.3. Must print the known checksum 9288233970. Run: python -I w3.py
import weakref

M = 100_000
state = 7


def rand():                                  # harness.h's h_rand()
    global state
    state = (state * 6364136223846793005 + 1442695040888963407) & 0xFFFF_FFFF_FFFF_FFFF
    return state >> 33


def sign(d):
    return (d > 0) - (d < 0)


class GameObject:
    __slots__ = ("id", "x", "y", "health", "_target", "__weakref__")

    def __init__(self, id, x, y, health):
        self.id = id; self.x = x; self.y = y; self.health = health
        self._target = None

    @property
    def target(self):                        # None once the target is gone
        return self._target() if self._target is not None else None

    @target.setter
    def target(self, t):
        self._target = weakref.ref(t) if t is not None else None

    def update(self):
        pass


class Ship(GameObject):
    __slots__ = ()
    def update(self):
        t = self.target
        if t is not None:
            self.x += sign(t.x - self.x)
            self.y += sign(t.y - self.y)


class Rock(GameObject):
    __slots__ = ()
    def update(self):
        self.x = (self.x + 1) % 1000


class Turret(GameObject):
    __slots__ = ()
    def update(self):
        t = self.target
        if t is not None and abs(t.x - self.x) + abs(t.y - self.y) < 300:
            t.health -= 1


KINDS = (Ship, Rock, Turret)


def make(id):
    x = rand() % 1000
    y = rand() % 1000
    return KINDS[id % 3](id, x, y, 100 + id % 50)


def main():
    objects = []
    for i in range(M):
        o = make(i)
        if i > 0:
            o.target = objects[rand() % i]
        objects.append(o)
    next_id = M
    destroyed = 0
    for frame in range(1000):
        for o in objects:
            o.update()
        before = len(objects)
        objects = [o for o in objects if o.health > 0]   # the dead die here, in id order
        dead = before - len(objects)
        destroyed += dead
        for _ in range(dead):
            o = make(next_id)
            next_id += 1
            if objects:
                o.target = objects[rand() % len(objects)]
            objects.append(o)
    print(sum(o.x + o.y + o.health for o in objects) + 1_000_000 * destroyed)


main()
