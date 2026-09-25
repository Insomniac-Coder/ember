#$ test: run-pass
#$ rules: EXP-2, TYP-21
#$ profiles: debug, release, shipping
#$ stdout: value index add_assign value index 3 4
# `[EXP-2]` — `a[i] += x` evaluates `x`, then the place once, then operates
# and writes: through `AddAssign.add_assign`, and through `a = a + x` where
# the type has only `Add` (`[TYP-21]`).

@derive(Copy)
struct A:
    n: int

extend A implements AddAssign:
    fn add_assign(mut self, rhs: A):
        print("add_assign ")
        self.n += rhs.n

@derive(Copy)
struct B:
    n: int

extend B implements Add:
    type Output = B

    fn add(self, rhs: B) -> B:
        return B(n=self.n + rhs.n)

fn index() -> int:
    print("index ")
    return 0

fn a_value() -> A:
    print("value ")
    return A(n=2)

fn b_value() -> B:
    print("value ")
    return B(n=3)

fn main():
    xs: Array[A] = [A(n=1)]
    xs[index()] += a_value()
    ys: Array[B] = [B(n=1)]
    ys[index()] += b_value()
    println(xs[0].n, ys[0].n)
