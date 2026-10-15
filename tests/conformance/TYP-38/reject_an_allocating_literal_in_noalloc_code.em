#$ test: compile-fail
#$ rules: TYP-38, HEAP-1, STD-14
#$ error[E4001]: `@noalloc` function `listed` reaches an allocation
# `[TYP-38]` — a list literal in an `Array` position allocates and carries `Alloc`, so in a
# `@noalloc` function it is `E4001`.

@noalloc
fn listed(n: int) -> int:
    xs = [n, n]
    return xs.len()

fn main():
    println(listed(1))
