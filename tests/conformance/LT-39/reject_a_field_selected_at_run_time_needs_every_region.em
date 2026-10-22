#$ test: compile-fail
#$ rules: LT-39, LT-21, LT-35
# `[LT-39]`: an operation that selects a field by a run-time value needs every region it could
# select, whichever one the value picks at run time. A conditional expression choosing `left` or
# `right` by a run-time value holds both regions (`[LT-21]`: a merge keeps every region that may
# reach), so ending `right`'s loan (by pushing to `right`) while the selection is still used is
# rejected even though the value picks `left`. `pick` does the same choice behind a call; its result
# holds every region of the `Pair` it is given (`[LT-1]`, `[LT-35]`).

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
    right: Array[i32] = Array[i32]()
    right.push(8)
    pair = bundle(left.as_span(), right.as_span())
    chosen = pick(pair, 0)
    right.push(9)    #$ error[E3021]: `right` is borrowed here and mutably borrowed elsewhere
    println(chosen[0])
    again = bundle(left.as_span(), right.as_span())
    which = 0
    selected = again.left if which == 0 else again.right
    right.push(10)    #$ error[E3021]: `selected` is what keeps it alive
    println(selected[0])
