#$ test: compile-fail
#$ rules: LT-24, LT-35, LT-36, LT-42, BRW-1, TST-19

# Reading a selected field permits the exact capture path, but returning the
# captured aggregate uses it as a whole value. That capture must retain both
# provenance slots, including `right`.

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
    copy_pair = fn() => pair
    right.push(9) #$ error[E3021]: `right` is borrowed here and mutably borrowed elsewhere
    println(copy_pair().left[0])
