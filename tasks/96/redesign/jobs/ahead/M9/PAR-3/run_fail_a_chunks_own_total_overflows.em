# Clause PAR-3.1: "A whole-number reduction's chunk results have the variable's type and stop the
# program on overflow, as `[THR-21]`'s task totals do (verdict 105): the result is the serial one
# whenever no chunk's own total overflows, and since chunk boundaries depend only on the trip count
# and `chunk`, whether it stops does not depend on the machine."
#$ test: run-fail
#$ rules: PAR-3, PAR-5, TYP-8
#$ panics: integer overflow
# With `chunk=2`, the chunks are `[-100, 0]` and `[100, 100]`; the second's own total, 200, does not
# fit an `i8`, though the serial sum is 100.

fn total_of(xs: Span[i8]) -> i8:
    t: i8 = 0
    @parallel(chunk=2, reduce=[t: +])
    for i in 0..xs.len():
        t += xs[i]
    return t

fn main():
    xs: Array[i8] = [-100, 0, 100, 100]
    println(total_of(xs))
