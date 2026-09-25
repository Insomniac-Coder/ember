#$ test: run-pass
#$ rules: IFC-3, STR-5
#$ stdout: 150 9
# D-307 — `interface Ord: Eq` requires `Eq`, and a struct has `Eq` field by
# field without writing it (`[STR-5]`). `implements Ord` refused `Money`
# until `Eq` was written out: the parent was looked for only among written
# `implements` lines.

@derive(Copy)
struct Money:
    cents: int

extend Money implements Ord:
    fn cmp(self, other: Money) -> Ordering:
        if self.cents < other.cents:
            return Ordering.Less
        if self.cents > other.cents:
            return Ordering.Greater
        return Ordering.Equal

fn biggest[T: Ord + Copy](a: T, b: T) -> T:
    if a.cmp(b) == Ordering.Greater:
        return a
    return b

fn main():
    println(biggest(Money(150), Money(90)).cents, biggest(3, 9))
