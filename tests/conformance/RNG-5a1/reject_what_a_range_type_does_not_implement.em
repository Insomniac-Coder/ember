#$ test: compile-fail
#$ rules: RNG-5a1, RNG-8, RNG-5
# N1, ODR-093 — what a range type does not have: `Default` (its
# representation's may be outside the range), an operator with another range
# type, an operator its representation lacks (an integer's `/`), and `Hash`
# when its representation has none.

type Unit = f64 in 0.0 ..= 1.0
type Other = f64 in 0.0 ..= 1.0
type Level = i32 in 0 ..= 10

fn zero[T: Default]() -> T:
    return T.default()

fn mix[B, A: Add[B]](a: A, b: B) -> A.Output:
    return a + b

fn divide[T: Div[T]](a: T, b: T) -> T.Output:
    return a / b

fn main():
    u = Unit.clamped(0.5)
    o = Other.clamped(0.5)
    l = Level.clamped(3)
    z: Unit = zero()                #$ error[E2040]: `Unit` does not implement `std.core.Default`
    m = mix(u, o)                   #$ error[E2040]: `Unit` does not implement `std.core.Add[Other]`
    d = divide(l, l)                #$ error[E2040]: `Level` does not implement `std.core.Div[Level]`
    keys: Map[Unit, int] = {}       #$ error[E2040]: `Unit` does not implement `std.collections.Hash`
    println(z, m, d, keys)
