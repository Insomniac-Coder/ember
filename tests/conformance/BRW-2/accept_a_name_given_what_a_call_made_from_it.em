#$ test: run-pass
#$ rules: BRW-2, FN-1, OWN-5
#$ stdout: 3
#$ stdout: 5
#$ stdout: 6
# `[BRW-2]` — a read-only view lasts until its last use: the views a call reads its arguments through end
# with the call, so a name may be given what a call made from it, for a type that is not `Copy` too.

struct Money:
    cents: int

    fn plus(other: Money) -> Money:
        return Money(this.cents + other.cents)

fn add(rhs: Money, b: Money) -> Money:
    return Money(rhs.cents + b.cents)

fn main():
    m = Money(1)
    two = Money(2)
    m = add(m, two)
    println(m.cents)
    m = m.plus(two)
    println(m.cents)
    held = [Money(1)]
    held[0] = add(held[0], Money(5))
    println(held[0].cents)
