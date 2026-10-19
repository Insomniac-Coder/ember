#$ test: run-pass
#$ rules: EFF-7
#$ stdout: 2
# `[EFF-7]` — `@assume_noalloc(call)` inside `unsafe` takes that call to carry no `Alloc`: the
# caller reserved room for every element, so `collect_visible` never grows `out`, and `cull`
# keeps its contract. The programmer answers for the claim, as for any `unsafe` obligation.

fn collect_visible(xs: Span[int], mut out: Array[int]):
    for x in xs:
        if x > 0:
            out.push(x)

@noalloc
fn cull(xs: Span[int], mut out: Array[int]) -> int:
    unsafe:  # SAFETY: the caller reserved room for every element, so the pushes never grow `out`
        @assume_noalloc(collect_visible(xs, out))
    return out.len()

fn main():
    values = [1, -2, 3]
    out: Array[int] = []
    out.reserve(3)
    println(cull(values.as_span(), out))
