#$ test: run-pass
#$ rules: LT-24, LT-35, LT-36, LT-42, TST-19
#$ stdout: 7

# A capturing closure is lowered to a direct call with its environment as the
# first argument. Its body reads only `pair.left`, so the environment summary
# must retain only the left provenance slot; mutating the unrelated `right`
# source before the closure call is valid.

@view
struct Pair:
    left: Span[i32]
    right: Span[i32]

fn bundle(left: Span[i32], right: Span[i32]) -> Pair:
    return Pair(left, right)

fn main():
    left: Array[i32] = Array[i32]()
    left.push(7)
    right: Array[i32] = Array[i32]()
    right.push(8)
    pair = bundle(left.as_span(), right.as_span())
    read_left = fn() => pair.left[0]
    right.push(9)
    println(read_left())
