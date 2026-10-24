# W1 (ECS) written the way a Python programmer writes a small registry: one dict per component type.
# The Python shown beside the Ember program in phase3b-combined.md section 6.1. (w1.py, written by
# another agent in this folder, is a line-for-line sparse-set twin of the Ember library instead.)
# Must print the known checksum 2111831520. Run: python -I w1_dicts.py
N = 1_000_000


class Position:
    __slots__ = ("x", "y")
    def __init__(self, x, y): self.x = x; self.y = y


class Velocity:
    __slots__ = ("dx", "dy")
    def __init__(self, dx, dy): self.dx = dx; self.dy = dy


class Registry:
    def __init__(self):
        self.pools = {}            # component class -> {entity: component}
        self.next_id = 0

    def create(self):
        e = self.next_id
        self.next_id += 1
        return e

    def add(self, e, component):
        self.pools.setdefault(type(component), {})[e] = component

    def get(self, e, T):
        return self.pools[T][e]

    def destroy(self, e):
        for pool in self.pools.values():
            pool.pop(e, None)

    def view(self, A, B):
        a, b = self.pools[A], self.pools[B]
        for e in list(a):                  # a copy of the keys: the loop body may add and destroy
            if e in a and e in b:          # skip what the body destroyed before its turn
                yield e, a[e], b[e]


def main():
    reg = Registry()
    for i in range(N):
        e = reg.create()
        reg.add(e, Position(float(i % 1000), float(i // 1000)))
        if i % 2 == 0:
            reg.add(e, Velocity((i % 7) * 0.5, (i % 5) * 0.25))
    for s in range(100):
        for e, pos, vel in reg.view(Position, Velocity):
            pos.x += vel.dx
            pos.y += vel.dy
            if e % 100 == s:
                reg.destroy(e)
                n = reg.create()
                reg.add(n, Position(0.0, 0.0))
                reg.add(n, Velocity(1.0, 1.0))
    positions = reg.pools[Position]
    checksum = sum(int(p.x * 4.0) + int(p.y * 4.0) for p in positions.values()) + len(positions)
    print(checksum)


main()
