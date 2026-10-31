#$ test: run-pass
#$ rules: CLO-14, CLO-11, CLO-3, TYP-16
#$ stdout: 30
#$ stdout: 15 8
# D-406, `[CLO-14]` — `F: fn(int) -> int` is the parameter form's bound
# written out: a parameter of type `F`, a local holding one, and a field of a
# generic struct over `F` (`[CLO-11]`) are called like a function. The bound
# was dropped, so each call was `E2020` (`F` cannot be called).

[F: needs fn(int) -> int]
fn apply(f: F, x: int) -> int:
    g = f
    return g(x) + f(x)

[F: needs fn(int) -> int]
struct Holder:
    f: F

    fn call(x: int) -> int:
        return this.f(x)

fn main():
    k = 10
    println(apply(fn(x: int) => x + k, 5))
    h = Holder(f = fn(x: int) => x + k)
    owned_h = Holder(f = owned fn(x: int) => x * 2)
    println(h.call(5), owned_h.call(4))
