#$ test: compile-pass
#$ rules: TST-25, SPN-4, SPN-5, SPN-6, SPN-7, SPN-8, SPN-9, SPN-10

from std.collections import SpanIter, SpanChunks

fn surface[T](span: Span[T]) -> usize:
    iterator: SpanIter[T] = span.iter()
    chunks: SpanChunks[T] = span.chunks(1)
    match iterator.next():
        Some(_item):
            pass
        None:
            pass
    match chunks.next():
        Some(chunk):
            return chunk.len()
        None:
            return 0

fn main():
    pass
