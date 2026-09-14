#$ test: run-pass
#$ rules: LT-14, LT-30, LT-31, LT-31a, TST-17
#$ assert-c-count: contains("int32_t em_left_value(") == 2
#$ assert-c: !contains("region_slot")
#$ stdout: 1
#$ stdout: 3

# The same nominal Pair and direct callable are used with two independent
# provenance vectors. The emitted C contains only its prototype and definition,
# not one symbol or layout per inferred region vector.

@view
struct Pair:
    left: Span[i32]
    right: Span[i32]

fn left_value(pair: Pair) -> i32:
    return pair.left[0]

fn main():
    first_left: Array[i32] = Array[i32]()
    first_left.push(1)
    first_right: Array[i32] = Array[i32]()
    first_right.push(2)
    second_left: Array[i32] = Array[i32]()
    second_left.push(3)
    second_right: Array[i32] = Array[i32]()
    second_right.push(4)
    first = Pair(first_left.as_span(), first_right.as_span())
    second = Pair(second_left.as_span(), second_right.as_span())
    println(left_value(first))
    println(left_value(second))
