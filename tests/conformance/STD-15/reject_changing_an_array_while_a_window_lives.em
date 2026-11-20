#$ test: compile-fail
#$ rules: STD-15, SPN-1, BRW-1
# `[STD-15]` (ODR-031) — a window is a view of the array (`[SPN-1]`): the array
# cannot change while a window, or the iterator making them, is alive.

fn main():
    xs = [1, 2, 3]
    for w in xs.windows(2):
        xs.push(owns w[0])                #$ error[E3020]: `xs.push(owns w[0])` (line 9) adds to `xs`, which this loop walks (line 8); collect what it adds in a list of your own: write `added: Array[int] = []` before the loop, `added.push(owns w[0])` in place of line 9, and `xs.extend(owns added)` after the loop
    it = xs.windows(2)
    xs.clear()
    _first = it.next()    #$ error[E3060]
