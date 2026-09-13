#$ test: run-pass
#$ rules: SPN-4, SPN-5, SPN-6, SPN-10, TST-25, TYP-16
#$ assert-c: contains("SpanIter_i32")
#$ assert-c: contains("MutSpanChunks_i32")

from std.collections import SpanIter, MutSpanIter, SpanChunks, MutSpanChunks

fn named_shared[T](span: Span[T]) -> usize:
    iterator: SpanIter[T] = span.iter()
    chunks: SpanChunks[T] = span.chunks(2)
    seen: usize = 0
    match iterator.next():
        Some(_item):
            seen = seen + 1
        None:
            pass
    match chunks.next():
        Some(_chunk):
            seen = seen + 1
        None:
            pass
    return seen

fn named_mut[T](mut span: MutSpan[T]) -> usize:
    iterator: MutSpanIter[T] = span.iter_mut()
    match iterator.next():
        Some(_item):
            return 1
        None:
            return 0

fn main():
    values: Array[i32] = Array[i32]()
    values.push(1)
    values.push(2)
    values.push(3)
    values.push(4)
    values.push(5)

    total: i32 = 0
    for item in values.as_span().iter():
        total = total + item
    println(total)

    shared = values.as_mut_span()
    shared_total: i32 = 0
    for item in shared.iter():
        shared_total = shared_total + item
    println(shared_total)

    mutable = values.as_mut_span()
    for item in mutable.iter_mut():
        replacement: i32 = item + 10
        item = ref mut replacement

    chunk_count: usize = 0
    last_len: usize = 0
    for chunk in values.as_span().chunks(2):
        chunk_count = chunk_count + 1
        last_len = chunk.len()
    println(chunk_count)
    println(last_len)

    mutable_chunks = values.as_mut_span()
    for chunk in mutable_chunks.chunks_mut(2):
        chunk[0] = chunk[0] + 100
    println(values[0])
    println(values[2])
    println(values[4])
    println(named_shared[i32](values.as_span()))
    println(named_mut[i32](values.as_mut_span()))
#$ stdout: 15
#$ stdout: 15
#$ stdout: 3
#$ stdout: 1
#$ stdout: 111
#$ stdout: 113
#$ stdout: 115
#$ stdout: 2
#$ stdout: 1
