#$ test: run-pass
#$ rules: TYP-17, TYP-21, IFC-4
#$ profiles: debug, development, release
#$ stdout: 6 0.75 V(x=4, y=6)
#$ -5 2.5 V(x=-1, y=-2)
# `[TYP-17]`'s own example: a bound that binds an associated type. The numbers
# implement `Add` and `Default` as a program's type does (ODR-040), so one
# `sum` takes all three, and inside it `total + x` is a `T`.

[T: needs (Add[Output = T], Default)]
fn sum(xs: Span[T]) -> T:
    total = T.default()
    for x in xs:
        total = total + x
    return total

[T: needs Neg[Output = T]]
fn negate(x: T) -> T:
    return -x

@derive(Copy)
struct V:
    x: int
    y: int

extend V implements Add, Neg, Default:
    type Output = V

    fn add(rhs: V) -> V:
        return V(x=this.x + rhs.x, y=this.y + rhs.y)

    fn neg() -> V:
        return V(x=-this.x, y=-this.y)

    static fn default() -> V:
        return V(x=0, y=0)

fn main():
    ints: Array[i32] = [1, 2, 3]
    floats: Array[f64] = [0.5, 0.25]
    vs: Array[V] = [V(x=1, y=2), V(x=3, y=4)]
    println(sum(ints.as_span()), sum(floats.as_span()), sum(vs.as_span()))
    println(negate(5), negate(-2.5), negate(V(x=1, y=2)))
