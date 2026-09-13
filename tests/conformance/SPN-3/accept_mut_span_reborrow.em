#$ test: run-pass
#$ rules: SPN-1, SPN-3, BRW-2, FN-1a
#$ stdout: 10
#$ stdout: 20
#$ stdout: 30
#$ stdout: 2

# Explicit reborrow produces a new mutable view without moving its parent.
# Direct view-producing expressions use the same operation and provenance.

fn view_len[T](mut span: MutSpan[T]) -> usize:
    child = span.reborrow()
    return child.len()

fn main():
    values: Array[i32] = Array[i32]()
    values.push(1)
    values.push(2)
    parent = values.as_mut_span()
    child = parent.reborrow()
    child[0] = 10
    println(child[0])
    parent[1] = 20
    println(parent[1])
    direct = values.as_mut_span().reborrow()
    direct[0] = 30
    println(direct[0])
    println(view_len[i32](values.as_mut_span()))
