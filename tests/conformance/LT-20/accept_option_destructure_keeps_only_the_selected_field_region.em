#$ test: run-pass
#$ rules: LT-17, LT-20, LT-24, LT-36, LT-38, TYP-15, TST-17
#$ stdout: 7

# Reading an enum discriminant does not read its payload. An explicit nested
# destructure moves only `Pair.left`; `Pair.right` is discarded and its region
# may end before the selected view is read.

@view
struct Pair:
    left: Span[i32]
    right: Span[i32]

fn main():
    left: Array[i32] = Array[i32]()
    left.push(7)
    right: Array[i32] = Array[i32]()
    right.push(8)
    maybe: Option[Pair] = Some(Pair(left.as_span(), right.as_span()))
    right.push(9)
    match maybe:
        Some(Pair(selected, _)):
            println(selected[0])
        None:
            pass
