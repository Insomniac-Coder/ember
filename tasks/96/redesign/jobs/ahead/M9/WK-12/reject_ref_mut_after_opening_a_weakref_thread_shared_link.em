# Clause WK-12.2 with `[THR-1]`: "no owner changes a `thread_shared` value directly" (E7014: "`ref
# mut` after `exists as` on a `weakref thread_shared T` link included")
#$ test: compile-fail
#$ rules: WK-12, THR-1
# No text in the spec for this program: code only.

class Stats:
    total: int = 0

fn main():
    st = thread_shared Stats()
    w = weakref st
    if w exists as ref mut s:    #$ error[E7014]
        s.total += 1
