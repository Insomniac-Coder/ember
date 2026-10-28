#$ test: run-pass
#$ rules: R12
#$ stdout: 1
#$ stdout: 2
#$ stdout: 1
## Verdict 80, R12 (rules.md lines 749-757; section 2, line 935, "nothing beyond the function value's own
## block" for a variable a function keeps with `with owns`): each counter keeps its own `t` in its own
## state, which outlives `make_counter` and ends with the function value; two counters never share one,
## and no box is made (completeness G10, settled by verdict 80). Checked by stdout, and by a build of the
## emitted C under AddressSanitizer with leak detection: no report.

struct Tally:
    n: int = 0

fn make_counter() -> fn():
    t = Tally()
    return fn() with owns t:
        t.n += 1
        println(t.n)

fn main():
    a = make_counter()
    b = make_counter()
    a()
    a()
    b()
