#$ test: compile-fail
#$ rules: TYP-17, STD-12
# `[TYP-17]` — a bound on a generic interface is checked with the call's
# arguments put in: `i32` implements `Keyed[i64]`, not `Keyed[String]`.

interface Keyed[K]:
    fn make_key(self) -> K

extend i32 implements Keyed[i64]:
    fn make_key(self) -> i64:
        return self as i64

fn conv[K, Q: Keyed[K]](q: Q) -> K:
    return q.make_key()

fn main():
    a: i64 = conv[i64, i32](1)
    b: String = conv[String, i32](2)  #$ error[E2040]: `i32` does not implement `Keyed[String]`, which `Q` requires
    println(a, b)
