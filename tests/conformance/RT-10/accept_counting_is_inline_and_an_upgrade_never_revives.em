#$ test: run-pass
#$ rules: RT-10, WK-12
#$ profiles: debug, development
#$ stdout: 2 4
#$ alive 1
#$ gone
#$ assert-c: contains(ember_retain_sync()
#$ assert-c: contains(ember_retain_plain()
#$ assert-c: contains(ember_weak_upgrade)
# `[RT-10]` — counting is inline: a copy of a `@sync` handle is the header's inline
# `ember_retain_sync` (one atomic add) and a copy of any other handle the inline
# `ember_retain_plain`, never a call into the runtime's own file. `Weak.upgrade` finds the
# object while a strong handle holds it, and finds it gone once the last one is released: it
# never raises a count from zero.

@sync
class Shared:
    value: int

class Local:
    value: int

fn program():
    s = Shared(1)
    copies = [s, s]
    p = Local(2)
    others = [p, p]
    println(copies[0].value + copies[1].value, others[0].value + others[1].value)
    weak: Weak[Shared] = Weak(s)
    copies.clear()
    match weak.upgrade():
        Some(still):
            println("alive", still.value)
        None:
            println("gone")
    s = Shared(3)
    match weak.upgrade():
        Some(still):
            println("alive", still.value)
        None:
            println("gone")

fn main():
    program()
