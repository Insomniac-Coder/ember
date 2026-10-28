#$ test: compile-fail
#$ rules: LT-45, BRW-19, ALC-1
#$ help: best = other
## design rules: R4, R6
## R4: inside the callee only a view of another view argument, a part of one, or text written in the
## program may be put into the caller's view through `ref mut`; a view of the callee's own local would
## outlive it (rules section 3, E3063).

fn pick_longer(ref mut best: Span[int], other: Span[int]):
    mine = [0, 0, 0, 0]
    best = mine.as_span()    #$ error[E3063]: `best` is the caller's view and lives on after `pick_longer` returns, but `mine` ends with `pick_longer`; put a view of what the caller gave there (`best = other`), or return a list of your own: `-> Array[int]`

fn main():
    a = [1]
    b = [1, 2, 3]
    best = a.as_span()
    pick_longer(ref mut best, b.as_span())
    println(best.len())
