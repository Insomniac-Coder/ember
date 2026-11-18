#$ test: run-pass
#$ rules: SPN-1, SPN-3, BRW-2, FN-1a
#$ stdout: 10
#$ stdout: 20
#$ stdout: 30
#$ stdout: 2

# Explicit reborrow produces a new mutable view without moving its parent.
# Direct view-producing expressions use the same operation and provenance.

[T]
fn view_len(ref mut span: MutSpan[T]) -> int:
    child = span.reborrow()
    return child.len()

fn main():
    values: Array[i32] = Array[i32]()
    values.push(owns 1)
    values.push(owns 2)
    parent = ref mut values.as_mut_span()
    child = ref mut parent.reborrow()
    child[0] = 10
    println(child[0])
    parent[1] = 20
    println(parent[1])
    direct = ref mut values.as_mut_span().reborrow()
    direct[0] = 30
    println(direct[0])
    println(view_len[i32](values.as_mut_span()))
