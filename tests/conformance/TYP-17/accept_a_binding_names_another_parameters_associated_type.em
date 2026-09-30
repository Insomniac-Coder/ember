#$ test: run-pass
#$ rules: TYP-17
#$ profiles: debug, release
#$ stdout:
#$ 1 2 10 20
#$ 3
# `[TYP-17]` / `[GRM-8c]` — a bound's binding may name an associated type of
# another of the declaration's parameters: `J: Iterator[Item = I.Item]` (D-389,
# which reported "`I` has no associated type `Item`"). And an interface's
# method may be bounded by that same interface with a binding: `Walk`'s
# `joined` (D-389, which overflowed the compiler's stack).

struct Both[I, J]:
    first: I
    other: J
    done: bool

extend[I: Iterator, J: Iterator[Item = I.Item]] Both[I, J] implements Iterator:
    type Item = I.Item

    fn next(mut self) -> Option[I.Item]:
        if not self.done:
            match self.first.next():
                Some(x):
                    return Some(x)
                None:
                    self.done = true
        return self.other.next()

fn both[I: Iterator, J: Iterator[Item = I.Item]](owned a: I, owned b: J) -> Both[I, J]:
    return Both(a, b, false)

interface Walk:
    type Step
    fn steps(self) -> int
    fn joined[W: Walk[Step = Step]](self, other: W) -> int:
        return self.steps() + other.steps()

struct Short:
    n: int

extend Short implements Walk:
    type Step = int

    fn steps(self) -> int:
        return self.n

fn main():
    xs: Array[int] = [1, 2]
    ys: Array[int] = [10, 20]
    parts: Array[String] = []
    for x in both(xs.iter(), ys.iter()):
        parts.push(f"{x}")
    println(parts.join(" "))
    println(Short(1).joined(Short(2)))
