# W1 (ECS) written in Python, the twin of the Ember registry and program in phase3b-combined.md
# section 6.1: the same sparse sets, the same full-owning group, the same cursor, line for line.
# Must print the known checksum 2111831520. Run: python -I w1.py   (slow: a few minutes)
NONE = 0xFFFF_FFFF
N = 1_000_000


class Pool:                                   # one sparse set per component type
    def __init__(self):
        self.sparse = []                      # entity -> slot, or NONE
        self.ents = []                        # slot -> entity
        self.dense = []                       # slot -> component
        self.cursor = 0                       # where a running view over this pool is

    def has(self, e):
        return e < len(self.sparse) and self.sparse[e] != NONE

    def add(self, e, value):
        while len(self.sparse) <= e:
            self.sparse.append(NONE)
        self.sparse[e] = len(self.ents)
        self.ents.append(e)
        self.dense.append(value)

    def swap(self, i, j):
        a, b = self.ents[i], self.ents[j]
        self.ents[i], self.ents[j] = b, a
        self.dense[i], self.dense[j] = self.dense[j], self.dense[i]
        self.sparse[b], self.sparse[a] = i, j

    def remove(self, e):                      # swap-and-pop, as EnTT
        i = self.sparse[e]
        if i < self.cursor:                   # a running view has not reached e yet:
            self.cursor -= 1                  # move the last unreached entity into e's slot,
            self.swap(i, self.cursor)         # then remove from that slot, a visited one
            i = self.cursor
        self.ents[i] = self.ents[-1]; self.ents.pop()
        self.dense[i] = self.dense[-1]; self.dense.pop()
        if i < len(self.ents):
            self.sparse[self.ents[i]] = i
        self.sparse[e] = NONE


class PairGroup:                              # EnTT's full-owning group of A and B
    def __init__(self, A, B):
        self.A, self.B = A, B
        self.glen = 0                         # entities with both sit in slots 0..glen-1 of both pools
        self.cursor = 0                       # where a running loop over the group is

    def entered(self, pools, e):              # after any add
        a, b = pools[self.A], pools[self.B]
        if a.has(e) and b.has(e) and a.sparse[e] >= self.glen:
            a.swap(a.sparse[e], self.glen)
            b.swap(b.sparse[e], self.glen)
            self.glen += 1

    def leaving(self, pools, e):              # before a destroy
        a, b = pools[self.A], pools[self.B]
        if not (a.has(e) and b.has(e)):
            return
        j = a.sparse[e]
        if j >= self.glen:
            return
        if j < self.cursor:                   # a running loop has not reached e yet
            self.cursor -= 1
            a.swap(j, self.cursor); b.swap(j, self.cursor)
            j = self.cursor
        self.glen -= 1
        a.swap(j, self.glen); b.swap(j, self.glen)


class Registry:
    def __init__(self):
        self.pools = {}                       # component class -> Pool
        self.groups = {}                      # (A, B) -> PairGroup
        self.next_id = 0

    def pool(self, T):
        return self.pools.setdefault(T, Pool())

    def create(self):
        e = self.next_id
        self.next_id += 1
        return e

    def get(self, T, e):                      # EnTT's registry.get<T>(e)
        pool = self.pool(T)
        return pool.dense[pool.sparse[e]]

    def add(self, e, value):
        self.pool(type(value)).add(e, value)
        for g in self.groups.values():
            g.entered(self.pools, e)

    def destroy(self, e):
        for g in self.groups.values():
            g.leaving(self.pools, e)
        for p in self.pools.values():
            if p.has(e):
                p.remove(e)

    def view(self, A, B):                     # EnTT's registry.view<A, B>(): walks B backwards
        a, b = self.pool(A), self.pool(B)
        outer = b.cursor
        try:
            k = len(b.ents)
            while k > 0:
                k -= 1
                e = b.ents[k]
                if a.has(e):
                    b.cursor = k
                    yield e, a.dense[a.sparse[e]], b.dense[k]
                    k = b.cursor
        finally:
            b.cursor = outer

    def make_group(self, A, B):
        self.pool(A); self.pool(B)
        self.groups[(A, B)] = PairGroup(A, B)

    def group(self, A, B):                    # EnTT's registry.group<A, B>(): the owned slots, backwards
        g, a, b = self.groups[(A, B)], self.pool(A), self.pool(B)
        outer = g.cursor
        try:
            k = g.glen
            while k > 0:
                k -= 1
                g.cursor = k
                yield a.ents[k], a.dense[k], b.dense[k]
                k = g.cursor
        finally:
            g.cursor = outer


class Position:
    __slots__ = ("x", "y")
    def __init__(self, x, y): self.x = x; self.y = y


class Velocity:
    __slots__ = ("dx", "dy")
    def __init__(self, dx, dy): self.dx = dx; self.dy = dy


def main():
    reg = Registry()
    reg.make_group(Position, Velocity)
    for i in range(N):
        e = reg.create()
        reg.add(e, Position(float(i % 1000), float(i // 1000)))
        if i % 2 == 0:
            reg.add(e, Velocity((i % 7) * 0.5, (i % 5) * 0.25))
    for s in range(100):
        for e, pos, vel in reg.group(Position, Velocity):
            pos.x += vel.dx
            pos.y += vel.dy
            if e % 100 == s:
                reg.destroy(e)
                n = reg.create()
                reg.add(n, Position(0.0, 0.0))
                reg.add(n, Velocity(1.0, 1.0))
    positions = reg.pool(Position)
    # get and view are exercised too: a view over (Position, Velocity) sees exactly the group's entities
    seen = [e for e, pos, vel in reg.view(Position, Velocity)]
    assert len(seen) == reg.groups[(Position, Velocity)].glen
    assert all(reg.get(Position, e) is positions.dense[positions.sparse[e]] for e in seen[:1000])
    print(sum(int(p.x * 4.0) + int(p.y * 4.0) for p in positions.dense) + len(positions.dense))


main()
