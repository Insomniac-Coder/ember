#$ test: compile-pass
#$ rules: SPN-4, SPN-10, MOD-2, TST-25

from std.collections import SpanIter, MutSpanIter, SpanChunks, MutSpanChunks

@view
struct PublicSurface:
    shared: SpanIter[i32]
    mutable: MutSpanIter[i32]
    chunks: SpanChunks[i32]
    chunks_mut: MutSpanChunks[i32]

fn main():
    pass
