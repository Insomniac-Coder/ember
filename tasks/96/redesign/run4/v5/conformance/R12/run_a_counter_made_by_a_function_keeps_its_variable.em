#$ test: run-pass
#$ rules: R12
#$ stdout: 1
#$ stdout: 2
#$ stdout: 1
## Verdicts 80 and 84, R12 (rules.md lines 761-767; section 2, line 985, "nothing beyond the function
## value's own block" for a variable a function keeps with `owns`): each counter keeps its own `n` in its
## own state, which outlives `make_counter` and ends with the function value; two counters never share
## one, and no box is made (completeness G10, settled by verdict 80). Checked by stdout, and by a build of
## the emitted C under AddressSanitizer with leak detection: no report.

fn make_counter() -> fn():
    n = 0
    return fn() owns n:
        n += 1
        println(n)

fn main():
    a = make_counter()
    b = make_counter()
    a()
    a()
    b()
