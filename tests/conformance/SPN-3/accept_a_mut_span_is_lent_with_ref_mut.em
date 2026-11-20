#$ test: run-pass
#$ rules: SPN-1, SPN-3, BRW-2, FN-1a
#$ stdout: 10
#$ stdout: 20
#$ stdout: 30
#$ stdout: 2

# `[SPN-3]` — a `MutSpan` is lent with `ref mut`, which before a span names its elements: a `ref mut` name
# of a `ref mut` name views the same elements without moving it, `ref mut` stands before a span made in
# the call, and a `ref mut` parameter is given `ref mut` before the argument.

[T]
fn view_len(ref mut span: MutSpan[T]) -> int:
    child = ref mut span
    return child.len()

fn main():
    values: Array[i32] = Array[i32]()
    values.push(owns 1)
    values.push(owns 2)
    parent = ref mut values.as_mut_span()
    child = ref mut parent
    child[0] = 10
    println(child[0])
    parent[1] = 20
    println(parent[1])
    direct = ref mut values.as_mut_span()
    direct[0] = 30
    println(direct[0])
    println(view_len[i32](ref mut values.as_mut_span()))
