# Python behaviour for the second containers attack, and a Python model of the answer's one-pass
# filter loop (A4 rules 1 to 4, with the logical-position mapping of the first attack's save).
# Run: C:/Python314/python.exe -I py_attack2.py
import copy

class E:
    def __init__(self, hp, name=None):
        self.hp, self.name = hp, name
    def __repr__(self):
        return f"E{self.hp}"

class Walk:
    """The answer's loop: physical list p, read position r, write position w; logical list is
    p[0:w] + p[r:] (p[r] left out once the current element is removed). Elements are values: the
    model keeps the current element `e` apart, as the compiled loop keeps it in registers."""
    def __init__(self, p):
        self.p, self.r, self.w, self.cur_removed = p, 0, 0, False
        self.end = None                        # snapshot mode: physical end of the original elements
    def phys(self, k):
        n = len(self.p) - (self.r - self.w) - self.cur_removed
        if not 0 <= k < n:
            raise IndexError("panic: index out of bounds")
        return k if k < self.w else k + (self.r - self.w) + self.cur_removed
    def delete(self, k):                       # del xs[k] in the body
        s = self.phys(k)
        if s == self.r:
            self.cur_removed = True
        elif s < self.w:
            self.p.pop(s); self.w -= 1; self.r -= 1
            if self.end is not None: self.end -= 1
        else:
            self.p.pop(s)
            if self.end is not None and s < self.end: self.end -= 1
    def run(self, body, index="count", snapshot=False):
        count = 0
        if snapshot:
            self.end = len(self.p)             # the saved copy loop: only the elements there at the start
        while self.r < (self.end if snapshot else len(self.p)):
            self.cur_removed = False
            e = copy.copy(self.p[self.r])      # e held apart (a value)
            i = count if index == "count" else self.w   # enumerate's index
            body(self, i, e)
            if not self.cur_removed:
                self.p[self.w] = e; self.w += 1
            self.r += 1; count += 1
        del self.p[self.w:self.r]                # close the gap (rule 6)
        return self.p

# 1. The Python habit for changing a list while walking it: walk a copy.
es = [E(1), E(3), E(1), E(5)]
seen = []
for e in es[:]:
    seen.append(e.hp)
    e.hp -= 1
    if e.hp <= 0:
        es.remove(e)
        es.append(E(9))
print("1 copy loop: visited", seen, "list", es)
assert seen == [1, 3, 1, 5] and repr(es) == "[E2, E4, E9, E9]"
# 1b. If the slice copied the elements themselves (struct values), the writes go to the copies.
es = [E(1), E(3), E(1), E(5)]
for e in copy.deepcopy(es):
    e.hp -= 1
print("1b value copy: the list after the loop", es)
assert repr(es) == "[E1, E3, E1, E5]"
# 1c. The answer's one-pass filter on the same body (appended enemies are visited).
def body1(wk, i, e):
    e.hp -= 1
    if e.hp <= 0:
        wk.cur_removed = True
        wk.p.append(E(9))
print("1c one-pass filter, appended visited:", Walk([E(1), E(3), E(1), E(5)]).run(body1))
out = Walk([E(1), E(3), E(1), E(5)]).run(body1, snapshot=True)
print("1d saved copy loop (the walk, bounded by the original elements):", out)
assert repr(out) == "[E2, E4, E9, E9]"

# 2. enumerate and del xs[i]: Python removes the element it visits (and skips the next one).
es = [E(1), E(1), E(5), E(1)]
for i, e in enumerate(es):
    e.hp -= 1
    if e.hp <= 0:
        del es[i]
print("2 Python enumerate + del:", es)
assert repr(es) == "[E1, E4]"
removed_alive = []
def body2(wk, i, e):
    e.hp -= 1
    if e.hp <= 0:
        s = wk.phys(i)
        if s != wk.r:
            removed_alive.append(wk.p[s])      # an enemy other than e
        wk.delete(i)
try:
    out = Walk([E(1), E(1), E(5), E(1)]).run(body2, index="count")
    print("2 answer, i = iteration count:", out)
except IndexError as x:
    print("2 answer, i = iteration count:", x, "after removing", removed_alive, "instead of e")
assert repr(removed_alive) == "[E5]"
out = Walk([E(1), E(1), E(5), E(1)]).run(body2, index="position")
print("2 answer, i = e's position now:", out)
assert repr(out) == "[E4]"

# 3. Writing the current element through the list by position.
es = [E(1), E(5)]
for e in es:
    e.hp -= 1
    es[0].hp += 10
print("3 write through es[0] in the loop:", es)
assert repr(es) == "[E20, E4]"
def body3(saved):
    def body(wk, i, e):
        e.hp -= 1
        s = wk.phys(0)
        if saved and s == wk.r:
            e.hp += 10                         # SAVED: the write goes to e itself
        else:
            wk.p[s].hp += 10                   # as written: to the slot, which e then overwrites
    return body
out = Walk([E(1), E(5)]).run(body3(False))
print("3 answer as written (only reads compared with r):", out)
assert repr(out) == "[E10, E4]"
out = Walk([E(1), E(5)]).run(body3(True))
print("3 saved (writes compared with r too):", out)
assert repr(out) == "[E20, E4]"

# 4. A slice passed to a function that grows every squad's list, the slice's own included.
class Squad:
    def __init__(self, units):
        self.units = units
def reinforce(squads, picked):
    for s in squads:
        s.units.extend(picked)
squads = [Squad([1, 2, 3]), Squad([4])]
reinforce(squads, squads[0].units[0:2])
print("4 reinforce:", [s.units for s in squads])
assert [s.units for s in squads] == [[1, 2, 3, 1, 2], [4, 1, 2]]

# 5. A slice kept across clear() through another name of the same squad.
def regroup(a, b):
    first_two = b.units[0:2]
    a.units.clear()
    return [u.name for u in first_two]
s = Squad([E(1, "ann"), E(2, "bob"), E(3, "cy")])
got = regroup(s, s)
print("5 slice across clear:", got, "squad now", s.units)
assert got == ["ann", "bob"] and s.units == []

# 6. A word kept across clear() of the word list.
text = "Alpha beta\nGamma delta"
words = []
for line in text.split("\n"):
    words.extend(line.lower().split())
best = words[0]
words.clear()
print("6 word kept across clear:", best)
assert best == "alpha"

# 7. The loop walks a list inside a list; other code replaces the outer element.
class World:
    def __init__(self):
        self.squads = [Squad([E(1), E(2), E(3)])]
w = World()
seen = []
for u in w.squads[0].units:
    seen.append(u.hp)
    if u.hp == 1:
        w.squads[0] = Squad([E(7)])        # what the callback does
print("7 nested walk, outer element replaced: visited", seen, "now", w.squads[0].units)
assert seen == [1, 2, 3] and repr(w.squads[0].units) == "[E7]"

print("all Python attack checks passed")
