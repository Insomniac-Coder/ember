# Clause LT-47.1: "only such a view may be put there: another view argument or a part of one, or
# text written in the program"; the message's fixes: "put a view of what the caller gave there
# (`best = other`), or return a list of your own: `-> Array[int]`"
#$ test: run-pass
#$ rules: LT-47, LT-1, DIA-25
#$ stdout: 2
#$ stdout: none
#$ stdout: 5
#$ stdout: 3
# A view argument, a part of one and program text are put in; the list of your own is returned;
# the put view is used before the argument changes.

fn pick_longer(ref mut best: Span[int], other: Span[int]):
    if other.len() > best.len():
        best = other

fn name_or_none(ref mut out: StrView, names: Span[String]):
    if names.len() > 0:
        out = names[0]
    else:
        out = "none"

fn longer_own(best: Span[int], other: Span[int]) -> Array[int]:
    mine = [0, 0, 0, 0]
    if other.len() > best.len():
        mine.push(owns 0)
    return mine

fn main():
    a = [1]
    b = [1, 2]
    best: Span[int] = a
    pick_longer(ref mut best, b)
    println(best.len())
    empty: Array[String] = []
    out: StrView = "x"
    name_or_none(ref mut out, empty)
    println(out)
    println(longer_own(a, b).len())
    bag = [1, 2, 3]
    v: Span[int] = a
    pick_longer(ref mut v, bag)
    println(v.len())
    bag.push(owns 4)
