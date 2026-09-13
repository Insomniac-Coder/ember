#$ test: run-pass
#$ rules: LT-30, LT-31, LT-31a, TST-17
#$ assert-c: contains("typedef struct em_Pair")
#$ assert-c: !contains("region_slot")
#$ stdout: 8

@view
struct Pair:
    left: Span[i32]
    right: Span[i32]

fn main():
    left: Array[i32] = Array[i32]()
    left.push(8)
    right: Array[i32] = Array[i32]()
    right.push(9)
    pair = Pair(left.as_span(), right.as_span())
    println(pair.left[0])
