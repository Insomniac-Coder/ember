#$ test: run-pass
#$ rules: IFC-3, IFC-4
#$ profiles: debug, release, shipping
#$ stdout: 3 2 1
#$ stdout: 30
# `[IFC-3]` — "a bound brings its parents", and so does an interface's own
# body: a child interface names its parent's associated type (`Item`), where
# the parent is std's `Iterator` or an interface later in the same module.
# D-394: a parent collected after its child was not known, and `Item` was
# "cannot find type".

interface Backwards: Iterator:
    fn next_back(mut self) -> Option[Item]

interface Weighed: Measured:
    fn heaviest(self) -> Unit

interface Measured:
    type Unit
    fn measure(self) -> Unit

struct Count:
    at: int
    end: int

extend Count implements Iterator:
    type Item = int

    fn next(mut self) -> Option[int]:
        if self.at < self.end:
            here = self.at
            self.at += 1
            return Some(here)
        return None

extend Count implements Backwards:
    fn next_back(mut self) -> Option[int]:
        if self.at < self.end:
            self.end -= 1
            return Some(self.end)
        return None

struct Crate:
    weight: int

extend Crate implements Measured:
    type Unit = int

    fn measure(self) -> int:
        return self.weight

extend Crate implements Weighed:
    fn heaviest(self) -> int:
        return self.weight * 3

fn main():
    c = Count(at = 1, end = 4)
    a = c.next_back()
    b = c.next_back()
    d = c.next_back()
    println(a.unwrap_or(0), b.unwrap_or(0), d.unwrap_or(0))
    println(Crate(weight = 10).heaviest())
