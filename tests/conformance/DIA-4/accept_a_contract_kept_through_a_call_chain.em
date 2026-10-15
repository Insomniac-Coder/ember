#$ test: run-pass
#$ rules: DIA-4, EFF-6, EFF-5
#$ stdout: 6
# `[DIA-4]` — a contract is checked through every function it reaches: `render` is `@noalloc` and
# its chain allocates nothing, so nothing is reported.

fn measure(xs: Span[int]) -> int:
    total = 0
    for x in xs:
        total += x
    return total

fn layout(xs: Span[int]) -> int:
    return measure(xs)

@noalloc
fn render(xs: Span[int]) -> int:
    return layout(xs)

fn main():
    values = [1, 2, 3]
    println(render(values))
