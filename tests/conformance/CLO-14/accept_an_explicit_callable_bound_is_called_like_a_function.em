#$ test: run-pass
#$ rules: CLO-14, CLO-11, CLO-3, TYP-16
#$ stdout: 30
#$ stdout: 15 8
# D-406, `[CLO-14]` — `F: fn(int) -> int` is the parameter form's bound
# written out: a parameter of type `F`, a local holding one, and a field of a
# generic struct over `F` (`[CLO-11]`) are called like a function. The bound
# was dropped, so each call was `E2020` (`F` cannot be called).

fn apply[F: fn(int) -> int](f: F, x: int) -> int:
    g = f
    return g(x) + f(x)

struct Holder[F: fn(int) -> int]:
    f: F

    fn call(self, x: int) -> int:
        return self.f(x)

fn main():
    k = 10
    println(apply(fn(x: int) => x + k, 5))
    h = Holder(f = fn(x: int) => x + k)
    owned_h = Holder(f = owned fn(x: int) => x * 2)
    println(h.call(5), owned_h.call(4))
