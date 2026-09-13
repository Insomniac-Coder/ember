#$ test: run-pass
#$ rules: SPN-4, TYP-17, IFC-4, TST-25

from std.core import Iterator

fn has_next[I: Iterator](mut iterator: I) -> bool:
    match iterator.next():
        Some(_) => return true
        None => return false

fn main():
    values: Array[i32] = Array[i32]()
    values.push(1)
    shared_iter = values.as_span().iter()
    println(has_next(shared_iter))
    shared_chunks = values.as_span().chunks(1)
    println(has_next(shared_chunks))
    mutable = values.as_mut_span()
    mutable_iter = mutable.iter_mut()
    println(has_next(mutable_iter))
    mutable_chunks = mutable.chunks_mut(1)
    println(has_next(mutable_chunks))
#$ stdout: true
#$ stdout: true
#$ stdout: true
#$ stdout: true
