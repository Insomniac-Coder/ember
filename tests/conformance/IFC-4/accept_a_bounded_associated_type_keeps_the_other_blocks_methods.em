#$ test: run-pass
#$ rules: IFC-4, TYP-17
#$ stdout: Some(2) Some(1)
# D-521 — whether an extension's method applies inside another generic body was asked of every
# parameter's bounds, a hidden one too (`I.Size: Ord`, from the interface's declaration), and read
# against that body's own list of parameters, where `I.Size` is not: `Peek[I]`'s `next`, from its
# `Seq` block, was "no method" in its other block. A hidden parameter's bounds are what its base's
# bound declares, met wherever the base's are.

interface Seq:
    type Item
    type Size: needs Ord = int
    fn next() -> Option[Item]

struct Down:
    n: int

extend Down implements Seq:
    type Item = int
    type Size = u64

    fn next() -> Option[int]:
        if this.n == 0:
            return None
        this.n -= 1
        return Some(this.n)

[I: needs Seq]
struct Peek:
    inner: I
    held: Option[I.Item]

extend[I: needs Seq] Peek[I] implements Seq:
    type Item = I.Item
    type Size = I.Size

    fn next() -> Option[I.Item]:
        return this.inner.next()

extend[I: needs Seq] Peek[I]:
    fn again() -> Option[I.Item]:
        return this.next()

fn main():
    p: Peek[Down] = Peek(Down(3), None)
    println(p.again(), p.again())
