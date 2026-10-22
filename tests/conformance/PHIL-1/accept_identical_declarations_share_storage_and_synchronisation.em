#$ test: run-pass
#$ rules: PHIL-1
#$ profiles: release
#$ stdout: 5
#$ 3
#$ 6
#$ assert-c: !contains("ember_release_plain((ember_obj_header*)")
#$ assert-c: !contains("ember_alloc(")
# `[PHIL-1]`: two syntactically identical declarations in one context have identical storage and
# synchronisation, decided by the declared type and never by how the program uses the value.
# `a` is passed `mut` to a function that writes it and `b` is only read, yet both are `Point`
# values in the frame: no heap allocation (`ember_alloc`) appears for either. `kept` is handed to a
# function that returns it and `alone` never leaves `main`, yet both are `@sync` objects whose
# counts are atomic: no plain (non-atomic) release is emitted for either.

struct Point:
    x: i64
    y: i64

fn nudge(mut p: Point):
    p.x += 1

@sync
class Tally:
    count: i64

fn keep(t: Tally) -> Tally:
    return t

fn main():
    a = Point(1, 2)
    b = Point(1, 2)
    nudge(a)
    println(a.x + a.y + b.x)
    kept = keep(Tally(1))
    alone = Tally(2)
    println(kept.count + alone.count)
    println(a.x + a.y + b.y)
