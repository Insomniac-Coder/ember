#$ test: run-pass
#$ rules: TYP-21
#$ stdout: 3 14
## `[TYP-21]` - `a += b` calls `AddAssign.add_assign` if the type implements it, else it is `a = a + b`.

@derive(Copy)
struct Money:
    cents: int

extend Money implements Add[Money]:
    type Output = Money

    fn add(rhs: Money) -> Money:
        return Money(this.cents + rhs.cents)

struct Counter:
    n: int

extend Counter implements AddAssign[int]:
    fn add_assign(rhs: int):
        this.n += rhs * 2

fn main():
    m = Money(1)
    m += Money(2)                    # no `AddAssign`: `m = m + Money(2)`
    c = Counter(4)
    c += 5                           # `add_assign`: 4 + 5 * 2
    println(m.cents, c.n)
