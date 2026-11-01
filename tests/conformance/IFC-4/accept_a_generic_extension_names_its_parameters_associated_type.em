#$ test: run-pass
#$ rules: IFC-4, TYP-17, GRM-34
#$ profiles: debug, development, release
#$ stdout: 1 2 3 | Some(1) None
# D-380 — a generic extension names its parameter's associated type,
# `type Item = I.Item` and `Option[I.Item]`, as a generic function's
# signature does (`[IFC-4]`). Applied to `Take[Count]` it is `Count`'s `Item`,
# `int`; inside a generic function over any `I: Iterator` it is that `I`'s.

from std.core import Iterator

[I]
struct Take:
    inner: I
    left: int

extend[I: needs Iterator] Take[I] implements Iterator:
    type Item = I.Item

    fn next() -> Option[I.Item]:
        if this.left <= 0:
            return None
        this.left -= 1
        return this.inner.next()

struct Count:
    n: int

extend Count implements Iterator:
    type Item = int

    fn next() -> Option[int]:
        this.n += 1
        return Some(this.n)

[I: needs Iterator]
fn first_of(owns it: I) -> Option[I.Item]:
    one = Take(it, 1)
    return one.next()

fn main():
    for x in Take(Count(0), 3):
        print(x, "")
    empty = Take(Count(0), 0)
    println("|", first_of(owns Count(0)), empty.next())
