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
    type Size: Ord = int
    fn next(mut self) -> Option[Item]

struct Down:
    n: int

extend Down implements Seq:
    type Item = int
    type Size = u64

    fn next(mut self) -> Option[int]:
        if self.n == 0:
            return None
        self.n -= 1
        return Some(self.n)

struct Peek[I: Seq]:
    inner: I
    held: Option[I.Item]

extend[I: Seq] Peek[I] implements Seq:
    type Item = I.Item
    type Size = I.Size

    fn next(mut self) -> Option[I.Item]:
        return self.inner.next()

extend[I: Seq] Peek[I]:
    fn again(mut self) -> Option[I.Item]:
        return self.next()

fn main():
    p: Peek[Down] = Peek(Down(3), None)
    println(p.again(), p.again())
