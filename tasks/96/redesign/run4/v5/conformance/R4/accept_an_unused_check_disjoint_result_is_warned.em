#$ test: compile-pass
#$ rules: R4
## R4, rules section 3's warnings: a `check_disjoint` result left unused proves nothing and is warned,
## with the lines to write, the import of `assert_disjoint` included, so the first way out compiles
## (W2190; completeness G3; step 3's re-check, wrong 3-6).

from std.mem import check_disjoint

fn main():
    a = [1.0, 2.0]
    b = [3.0]
    check_disjoint(a.as_span(), b.as_span())    #$ warning[W2190]: to stop when they overlap, write `assert_disjoint(a.as_span(), b.as_span())`, with `from std.mem import assert_disjoint` at the top; to handle both cases, write `match check_disjoint(a.as_span(), b.as_span()):`
    println(a.len())
