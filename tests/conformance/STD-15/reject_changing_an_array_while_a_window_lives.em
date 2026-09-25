#$ test: compile-fail
#$ rules: STD-15, SPN-1, BRW-1
# `[STD-15]` (ODR-031) — a window is a view of the array (`[SPN-1]`): the array
# cannot change while a window, or the iterator making them, is alive.

fn main():
    xs = [1, 2, 3]
    for w in xs.windows(2):
        xs.push(w[0])                #$ error[E3020]: cannot mutate `xs` while it is borrowed by this loop
    it = xs.windows(2)
    xs.clear()                       #$ error[E3021]
    _first = it.next()
