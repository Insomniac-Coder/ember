#$ test: run-pass
#$ rules: TYP-38, TXT-9
#$ profiles: debug, release, shipping
#$ stdout: 4 10
#$ 2 bob
#$ 0
#$ assert-c: contains("ember_vec_from_elems(sizeof(int64_t)")
# A list literal with no context, or where an `Array[T]` is expected, is an
# `Array`: one allocation holding the elements, which can then grow. Each
# element converts to the element type, so string literals become `String`s.
# `[]` where an `Array` is expected is an empty one.

fn total(xs: Span[int]) -> int:
    sum = 0
    for i in 0..xs.len():
        sum += xs[i]
    return sum

fn main():
    xs = [1, 2, 3]
    xs.push(4)
    println(xs.len(), total(xs))
    names: Array[String] = ["ann", "bob"]
    println(names.len(), names[1].as_str())
    empty: Array[int] = []
    println(empty.len())
