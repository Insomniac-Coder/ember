#$ test: run-pass
#$ rules: PHIL-4, RC-4
#$ profiles: debug, development
#$ stdout: 6
#$ assert-c: contains(ember_retain_plain()
#$ assert-c: !contains(ember_retain_sync)
#$ assert-c: !contains(ember_release_sync)
#$ assert-c: !contains(__atomic)
#$ assert-c: !contains(_Interlocked)
# `[PHIL-4]` — no implicit synchronisation: a class that is not `@sync` counts its handles with
# plain integers, and nothing in the program takes a lock or an atomic operation, because no type
# in it is documented to synchronise.

class Counter:
    n: int

fn main():
    c = Counter(3)
    copies = [c, c]
    println(copies[0].n + copies[1].n)
