#$ test: run-pass
#$ rules: RT-10, WK-12
#$ profiles: debug, development
#$ stdout: 2 4
#$ alive 1
#$ gone
#$ assert-c: contains(ember_retain_sync()
#$ assert-c: contains(ember_retain_plain()
#$ assert-c: contains(ember_weak_upgrade)
# `[RT-10]` — counting is inline: one more owner of a `thread_shared` object is the header's inline
# `ember_retain_sync` (one atomic add) and one of any other object the inline
# `ember_retain_plain`, never a call into the runtime's own file. `Weak.upgrade` finds the
# object while a strong handle holds it, and finds it gone once the last one is released: it
# never raises a count from zero.

class Shared:
    value: int

class Local:
    value: int

fn program():
    s = thread_shared Shared(2)
    copies = [thread_shared s, thread_shared s]
    p = shares Local(1)
    others = [shares p, shares p]
    println(others[0].value + others[1].value, copies[0].value + copies[1].value)
    weak: weakref Local = weakref p
    others.clear()
    if weak exists as still:
        println("alive", still.value)
    else:
        println("gone")
    p = shares Local(3)
    if weak exists as still:
        println("alive", still.value)
    else:
        println("gone")

fn main():
    program()
