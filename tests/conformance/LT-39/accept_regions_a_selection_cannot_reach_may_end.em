#$ test: run-pass
#$ rules: LT-39, LT-24, LT-20
#$ stdout: 7
#$ 5
#$ 7
# `[LT-39]`'s other side: a selection needs the regions it *could* select, and no more and no
# longer. A run-time choice between two parts of `left` (`[LT-20]`: projecting a field carries only
# that field's region) leaves `right` free to end while its result is still used; and once a
# selection that could reach `right` is last used, every region it held ends with it (`[LT-24]`), so
# `right` may be pushed to afterwards. (A function's result is tied by its signature, `[LT-1]`, not
# by what its body selects: `pick` below ties its result to the whole `Pair`.)

@view
struct Pair:
    left: Span[i32]
    right: Span[i32]

fn bundle(left: Span[i32], right: Span[i32]) -> Pair:
    return Pair(left, right)

fn pick(pair: Pair, which: int) -> Span[i32]:
    if which == 0:
        return pair.left
    return pair.right

fn main():
    left: Array[i32] = Array[i32]()
    left.push(7)
    left.push(5)
    right: Array[i32] = Array[i32]()
    right.push(8)
    pair = bundle(left.as_span(), right.as_span())
    which = 0
    head = pair.left if which == 0 else pair.left[1..]
    tail = pair.left[1..] if which == 0 else pair.left
    right.push(9)
    println(head[0])
    println(tail[0])
    again = bundle(left.as_span(), right.as_span())
    chosen = pick(again, 0)
    println(chosen[0])
    right.push(10)
