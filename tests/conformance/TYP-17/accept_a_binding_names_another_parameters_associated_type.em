#$ test: run-pass
#$ rules: TYP-17
#$ profiles: debug, development
#$ stdout:
#$ 1 2 10 20
#$ 3
# `[TYP-17]` / `[GRM-8c]` — a bound's binding may name an associated type of
# another of the declaration's parameters: `J: Iterator[Item = I.Item]` (D-389,
# which reported "`I` has no associated type `Item`"). And an interface's
# method may be bounded by that same interface with a binding: `Walk`'s
# `joined` (D-389, which overflowed the compiler's stack).

[I, J]
struct Both:
    first: I
    other: J
    done: bool

extend[I: needs Iterator, J: needs Iterator[Item = I.Item]] Both[I, J] implements Iterator:
    type Item = I.Item

    fn next() -> Option[I.Item]:
        if not this.done:
            match this.first.next():
                Some(x):
                    return Some(x)
                None:
                    this.done = true
        return this.other.next()

[I: needs Iterator, J: needs Iterator[Item = I.Item]]
fn both(owns a: I, owns b: J) -> Both[I, J]:
    return Both(a, b, false)

interface Walk:
    type Step
    fn steps() -> int
    [W: needs Walk[Step = Step]]
    fn joined(other: W) -> int:
        return this.steps() + other.steps()

struct Short:
    n: int

extend Short implements Walk:
    type Step = int

    fn steps() -> int:
        return this.n

fn main():
    xs: Array[int] = [1, 2]
    ys: Array[int] = [10, 20]
    parts: Array[String] = []
    for x in both(owns xs.iter(), owns ys.iter()):
        parts.push(owns f"{x}")
    println(parts.join(" "))
    println(Short(1).joined(Short(2)))
