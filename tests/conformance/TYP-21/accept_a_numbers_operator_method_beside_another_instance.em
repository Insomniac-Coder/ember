#$ test: run-pass
#$ rules: TYP-21, TYP-24
#$ stdout: 1 1 3 3
# `[TYP-21]` — a number's operator method is its built-in operator, and an
# instance of the operator's interface over another right-hand type does not
# take it away: with `i64` implementing `Rem[Steps]`, `x.rem(y)` and
# `7.rem(2)` are still `%` on two `i64`s, and `x.rem(Steps(4))` is the
# instance's (D-338: the instance took every call).

struct Steps:
    n: i64

extend i64 implements Rem[Steps]:
    type Output = i64

    fn rem(self, s: Steps) -> i64:
        return self % s.n

fn main():
    x: i64 = 7
    y: i64 = 2
    println(x.rem(y), 7.rem(2), x.rem(Steps(4)), x % Steps(4))
