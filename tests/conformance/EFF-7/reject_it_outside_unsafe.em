#$ test: compile-fail
#$ rules: EFF-7
#$ error[E0104]: `@assume_noalloc` is admitted only inside `unsafe`
# `[EFF-7]` — the override is an `unsafe` obligation, so it is admitted only inside `unsafe`.

fn collect_visible(xs: Span[int], mut out: Array[int]):
    for x in xs:
        if x > 0:
            out.push(x)

@noalloc
fn cull(xs: Span[int], mut out: Array[int]) -> int:
    @assume_noalloc(collect_visible(xs, out))
    return out.len()

fn main():
    values = [1, -2, 3]
    out: Array[int] = []
    out.reserve(3)
    println(cull(values.as_span(), out))
