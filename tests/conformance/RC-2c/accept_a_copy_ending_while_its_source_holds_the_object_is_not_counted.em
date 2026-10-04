#$ test: run-pass
#$ rules: RC-2c, RC-2, RC-3, RC-1
#$ profiles: debug, release, shipping
#$ assert-c-count: contains("ember_retain_plain((ember_obj_header*)") == 1
#$ assert-c-count: contains("ember_retain((ember_obj_header*)") == 0
#$ stdout: 3 15
#$ stdout: 5
#$ stdout: 1 2
#$ stdout: drop 1
#$ stdout: drop 2
#$ stdout: drop 5
# `[RC-2c]` — `b = a` retains and `b`'s end releases; with nothing between
# them that could end the object (`a` keeps its own count and is not
# re-pointed, moved or dropped while `b` lives) the pair cancels (D-506).
# In `replaced` the source is assigned anew while `kept` still holds the
# old object, so that copy keeps its retain (the one counted above): the
# old object ends with `kept`, not with the assignment.

class Leaf:
    v: int

    fn drop(mut self):
        println("drop", self.v)

fn total_of(leaf: Leaf) -> int:
    return leaf.v

fn replaced():
    a = Leaf(1)
    kept = a
    a = Leaf(2)
    println(kept.v, a.v)

fn main():
    a = Leaf(5)
    total = 0
    count = 0
    for i in 0..3:
        b = a
        total += total_of(b)
        count += 1
    println(count, total)
    println(a.v)
    replaced()
