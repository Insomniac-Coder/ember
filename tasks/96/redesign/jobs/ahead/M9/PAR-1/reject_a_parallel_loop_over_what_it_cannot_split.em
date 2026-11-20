# Clause PAR-1.1: "A `@parallel` loop splits a range (`for i in 0..n:`) or a list walked by element
# (`for ref mut p in players:`) into chunks (...). Over anything else it is `E7016`"
#$ test: compile-fail
#$ rules: PAR-1, DIA-25
# The spec's message, with this file's line number.

fn place(ref mut out: Array[int], idx: Span[int]):
    i = 0
    @parallel
    while i < idx.len():    #$ error[E7016]: `@parallel` splits a range or a list into chunks, and `while i < idx.len():` (line 10) in `place` is neither; instead of `i = 0`, the `while` line and `i += 1`, write `for i in 0..idx.len():`
        out[i] = idx[i]
        i += 1

fn main():
    out = [0, 0]
    place(ref mut out, [1, 2])
