#$ test: parse-fail
#$ rules: GRM-23
# "non-associative: `a in b in c` is `E0104`". Ember has no chained
# comparison, and the Python reading would surprise a C++ programmer.
#
# Note the code: Part III §5's table gives `E0102` for a chained *comparison*,
# and `[GRM-23]` names `E0104` for this. The document is explicit, so the
# membership operators report `E0104` (errata ERR-026).

fn main():
    xs: Array[i32] = Array[i32]()
    ys: Array[i32] = Array[i32]()
    b = 1 in xs in ys           #$ error[E0104]: chained membership
    println(1)
