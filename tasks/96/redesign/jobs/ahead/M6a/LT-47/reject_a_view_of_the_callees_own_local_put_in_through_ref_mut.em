# Clause LT-47.1: "Inside the callee only such a view may be put there: another view argument or a
# part of one, or text written in the program; a view of the callee's own locals or of a temporary
# is `E3063`"
#$ test: compile-fail
#$ rules: LT-47, LT-1, BCK-4
# The spec's message, with this file's names.

fn pick_longer(ref mut best: Span[int], other: Span[int]):
    mine = [0, 0, 0, 0]
    if other.len() > best.len():
        best = other
    else:
        best = mine    #$ error[E3063]: `best` is the caller's view and lives on after `pick_longer` returns, but `mine` ends with `pick_longer`; put a view of what the caller gave there (`best = other`), or return a list of your own: `-> Array[int]`

fn main():
    a = [1]
    b = [1, 2]
    best: Span[int] = a
    pick_longer(ref mut best, b)
    println(best.len())
