#$ test: run-pass
#$ rules: TYP-17, TYP-21, TYP-40, IFC-4
#$ profiles: debug, development, release
#$ stdout: 5 9 13 17
# Operator bounds and ordinary method bounds both work when called with
# concrete values. A lookalike is callable by its method name, not as `+`.

interface Lookalike:
    fn add(rhs: Self) -> Self

@derive(Copy)
struct Ordinary:
    value: int

extend Ordinary implements Lookalike:
    fn add(rhs: Ordinary) -> Ordinary:
        return Ordinary(this.value + rhs.value)

@derive(Copy)
struct Sum:
    value: int

extend Sum implements Add:
    type Output = Sum
    fn add(rhs: Sum) -> Sum:
        return Sum(this.value + rhs.value)

interface Holder:
    type Item: needs Add[int]

struct Ints:
    unused: int

extend Ints implements Holder:
    type Item = int

[T: needs Lookalike]
fn named(a: T, b: T) -> T:
    return a.add(b)

[T: needs Add[Output = T]]
fn direct(a: T, b: T) -> T:
    return a + b

[H: needs Holder]
fn projected(holder: H, a: H.Item, b: int) -> H.Item.Output:
    return a + b

fn main():
    println(named(Ordinary(2), Ordinary(3)).value,
            direct(4, 5), direct(Sum(6), Sum(7)).value,
            projected(Ints(0), 8, 9))
