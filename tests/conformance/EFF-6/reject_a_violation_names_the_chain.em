#$ test: compile-fail
#$ rules: EFF-6
#$ error[E4001]: cull -> collect_visible -> `Array.push`
# `[EFF-6]` — a contract is checked over the whole reachable call graph, and a violation names
# the chain from the function down to the offending operation.

fn collect_visible(xs: Span[int], mut out: Array[int]):
    for x in xs:
        if x > 0:
            out.push(x)

@noalloc
fn cull(xs: Span[int], mut out: Array[int]) -> int:
    collect_visible(xs, out)
    return out.len()

fn main():
    values = [1, -2, 3]
    out: Array[int] = []
    println(cull(values.as_span(), out))
