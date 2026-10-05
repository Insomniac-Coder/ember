#$ test: run-pass
#$ rules: IFC-4, IFC-3
#$ stdout: 3 2
# D-518 — each `extend` block of a generic type makes its own hidden parameter for `I.Size`,
# numbered in that block's order: `Wrap[I]`'s `Seq` block has `I.Item` before it, its `Measured`
# block only `I.Size`. Its `size` gives that block's `I.Size`, and `Measured` asks for `Self.Size`,
# which is the other block's: one type, compared as two, so the implementation was refused.

interface Seq:
    type Item
    type Size = int
    fn next(mut self) -> Option[Item]

interface Measured: Seq:
    fn size(self) -> Size

struct Wrap[I]:
    inner: I

extend[I: Seq] Wrap[I] implements Seq:
    type Item = I.Item
    type Size = I.Size

    fn next(mut self) -> Option[I.Item]:
        return self.inner.next()

extend[I: Measured] Wrap[I] implements Measured:
    fn size(self) -> I.Size:
        return self.inner.size()

struct Down:
    n: int

extend Down implements Seq:
    type Item = int

    fn next(mut self) -> Option[int]:
        if self.n == 0:
            return None
        self.n -= 1
        return Some(self.n)

extend Down implements Measured:
    fn size(self) -> int:
        return self.n

fn main():
    w = Wrap(Down(3))
    before = w.size()
    w.next()
    println(before, w.size())
