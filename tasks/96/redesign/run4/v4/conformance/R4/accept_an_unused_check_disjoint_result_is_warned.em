#$ test: compile-pass
#$ rules: R4
## R4, rules section 3's warnings: a `check_disjoint` result left unused proves nothing and is warned,
## with the two lines to write (W2190; completeness G3).

from std.mem import check_disjoint

fn main():
    a = [1.0, 2.0]
    b = [3.0]
    check_disjoint(a.as_span(), b.as_span())    #$ warning[W2190]: to stop when they overlap, write `assert_disjoint(a.as_span(), b.as_span())`; to handle both cases, write `match check_disjoint(a.as_span(), b.as_span()):`
    println(a.len())
