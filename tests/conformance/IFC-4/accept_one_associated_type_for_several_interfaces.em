#$ test: run-pass
#$ rules: IFC-4, TYP-17, TYP-21
#$ profiles: debug, release, shipping
#$ stdout: 5 -1 6
#$ {1, 2, 3} {2} {1}
# ODR-040 — an `extend` block that implements several interfaces states an
# associated type once, for each of them that declares it; a generic
# extension's is its instance's (`Set[int]`'s `|`, `&` and `-`).

@derive(Copy)
struct N:
    v: int

extend N implements Add, Sub, Mul:
    type Output = int

    fn add(self, rhs: N) -> int:
        return self.v + rhs.v

    fn sub(self, rhs: N) -> int:
        return self.v - rhs.v

    fn mul(self, rhs: N) -> int:
        return self.v * rhs.v

fn apply[T: Add[Output = int] + Sub[Output = int] + Mul[Output = int] + Copy](a: T, b: T) -> (int, int, int):
    return (a + b, a - b, a * b)

fn main():
    (s, d, p) = apply(N(v=2), N(v=3))
    println(s, d, p)
    x: Set[int] = {1, 2}
    y: Set[int] = {2, 3}
    println(x | y, x & y, x - y)
