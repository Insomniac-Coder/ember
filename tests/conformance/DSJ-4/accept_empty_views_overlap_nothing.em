#$ test: run-pass
#$ rules: DSJ-4, DSJ-1
#$ profiles: debug, release, shipping
#$ stdout: apart 0 3

# `[DSJ-4]` — `Span` and `MutSpan` have address ranges; an empty view has no
# bytes, so it overlaps nothing, even one it sits inside.

from std.mem import assert_disjoint

fn main():
    x: Array[int] = [1, 2, 3]
    match assert_disjoint(x[1..1], x[0..3]):
        Ok((e, all)):
            println("apart", e.len(), all.len())
        Err((e, all)):
            println("overlap", e.len(), all.len())
