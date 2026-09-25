#$ test: run-pass
#$ rules: STD-16, STD-8
#$ stdout: true false 2 true false
#$ stdout: {1, 2, 3, 4} {2, 3} {1} {1, 4}
#$ stdout: true true false true
#$ stdout: true true {1, 3}
#$ stdout: Some(3) 1
#$ stdout: set() [1, 2, 3]
# `[STD-16]` (ODR-032) — the `Set` operations: `add` says whether the
# element was new, the set operations keep this set's order and then the
# other's, and `==` compares as sets.

fn main():
    s: Set[int] = {1, 2}
    println(s.add(3), s.add(3), s.len() - 1, 3 in s, 9 in s)
    t: Set[int] = {2, 3, 4}
    println(s | t, s & t, s - t, s ^ t)
    println({2, 3}.is_subset(s), s.is_superset({1, 2}), s.is_disjoint(t), {9}.is_disjoint(t))
    removed = s.remove(2)
    again = s.remove(2)
    println(s == {3, 1}, removed and not again, s)
    println(s.pop(), len(s))
    s.clear()
    println(s, sorted({3, 1, 2}))
