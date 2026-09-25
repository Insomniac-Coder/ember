#$ test: run-pass
#$ rules: TYP-17, TYP-21, IFC-4
#$ profiles: debug, release, shipping
#$ stdout: 6 0.75 V(x=4, y=6)
#$ -5 2.5 V(x=-1, y=-2)
# `[TYP-17]`'s own example: a bound that binds an associated type. The numbers
# implement `Add` and `Default` as a program's type does (ODR-040), so one
# `sum` takes all three, and inside it `total + x` is a `T`.

fn sum[T: Add[Output = T] + Default](xs: Span[T]) -> T:
    total = T.default()
    for x in xs:
        total = total + x
    return total

fn negate[T: Neg[Output = T]](x: T) -> T:
    return -x

@derive(Copy)
struct V:
    x: int
    y: int

extend V implements Add, Neg, Default:
    type Output = V

    fn add(self, rhs: V) -> V:
        return V(x=self.x + rhs.x, y=self.y + rhs.y)

    fn neg(self) -> V:
        return V(x=-self.x, y=-self.y)

    fn default() -> V:
        return V(x=0, y=0)

fn main():
    ints: Array[i32] = [1, 2, 3]
    floats: Array[f64] = [0.5, 0.25]
    vs: Array[V] = [V(x=1, y=2), V(x=3, y=4)]
    println(sum(ints.as_span()), sum(floats.as_span()), sum(vs.as_span()))
    println(negate(5), negate(-2.5), negate(V(x=1, y=2)))
