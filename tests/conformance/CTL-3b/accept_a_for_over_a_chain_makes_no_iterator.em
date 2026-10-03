#$ test: run-pass
#$ rules: CTL-3b, SIMD-5, SIMD-7
#$ profiles: debug, release
#$ stdout: 36
#$ assert-c-count: contains("Chain_std_collections_SpanIter_i64_std_collections_SpanIter_i64_next") == 0
#$ assert-c-count: contains("for (;") == 4
# `[CTL-3b]` — a `for` over `a.chain(b)` of two views never runs `Chain`'s
# `next`, nor calls anything per element: it is two counted loops, one per
# part (ADR-083). Each part also has an unchecked copy guarded by SIMD-7's
# block proof, now that its read-only element references are understood.

fn main():
    xs: Array[int] = [1, 2, 3]
    ys: Array[int] = [10, 20]
    total = 0
    for x in xs.iter().chain(ys.iter()):
        total += x
    println(total)
