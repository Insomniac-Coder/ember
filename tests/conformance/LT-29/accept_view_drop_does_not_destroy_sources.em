#$ test: run-pass
#$ rules: LT-18, LT-26, LT-29, OWN-2, TST-17
#$ assert-c-count: contains("ember_vec_free(") == 2
#$ stdout: 7

# The Pair is a non-owning view. At scope exit only its two Array sources are
# freed; dropping Pair itself is region/loan bookkeeping only and emits no
# source destruction or retention operation.

@view
struct Pair:
    left: Span[i32]
    right: Span[i32]

fn main():
    left: Array[i32] = Array[i32]()
    left.push(7)
    right: Array[i32] = Array[i32]()
    right.push(8)
    pair = Pair(left.as_span(), right.as_span())
    println(pair.left[0])
