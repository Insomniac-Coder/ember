#$ test: compile-fail
#$ rules: EFF-7
#$ error[E4001]: cull -> collect_visible -> `Array.push`
# `[EFF-7]` — the override covers that one call: the second call of `collect_visible`, not
# written `@assume_noalloc(…)`, still carries `Alloc` and breaks the contract.

fn collect_visible(xs: Span[int], mut out: Array[int]):
    for x in xs:
        if x > 0:
            out.push(x)

@noalloc
fn cull(xs: Span[int], mut out: Array[int]) -> int:
    unsafe:  # SAFETY: the caller reserved room for every element, so the pushes never grow `out`
        @assume_noalloc(collect_visible(xs, out))
    collect_visible(xs, out)
    return out.len()

fn main():
    values = [1, -2, 3]
    out: Array[int] = []
    out.reserve(6)
    println(cull(values.as_span(), out))
