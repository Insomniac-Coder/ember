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
    fn next() -> Option[Item]

interface Measured: needs Seq:
    fn size() -> Size

[I]
struct Wrap:
    inner: I

extend[I: needs Seq] Wrap[I] implements Seq:
    type Item = I.Item
    type Size = I.Size

    fn next() -> Option[I.Item]:
        return this.inner.next()

extend[I: needs Measured] Wrap[I] implements Measured:
    fn size() -> I.Size:
        return this.inner.size()

struct Down:
    n: int

extend Down implements Seq:
    type Item = int

    fn next() -> Option[int]:
        if this.n == 0:
            return None
        this.n -= 1
        return Some(this.n)

extend Down implements Measured:
    fn size() -> int:
        return this.n

fn main():
    w = Wrap(Down(3))
    before = w.size()
    w.next()
    println(before, w.size())
