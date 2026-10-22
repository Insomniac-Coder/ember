#$ test: run-pass
#$ rules: VER-3, GRM-34
#$ profiles: debug
#$ stdout: 2 2 4 3 5
## `[VER-3]` (ODR-174) — `W2192` for every kind of item: an interface named in an `implements`
## clause, a bound and a `dyn` type; a `static`; a type alias; an associated function.

from support.old import Sized2, SEEN, Count, Maker

struct Rod:
    n: int

extend Rod implements Sized2:    #$ warning[W2192]: `support.old.Sized2` is deprecated since 0.9: use `Measured`
    fn size(self) -> int:
        return self.n

fn measure[T: Sized2](t: T) -> int:    #$ warning[W2192]: `support.old.Sized2` is deprecated since 0.9
    return t.size()

fn measure_dyn(t: ref dyn Sized2) -> int:    #$ warning[W2192]: `support.old.Sized2` is deprecated since 0.9
    return t.size()

fn main():
    r = Rod(2)
    c: Count = 3    #$ warning[W2192]: `support.old.Count` is deprecated since 0.9: write `int`
    m = Maker.make(5)    #$ warning[W2192]: `support.old.Maker.make` is deprecated since 0.9: use `Maker(v)`
    println(measure(r), measure_dyn(ref r), SEEN, c, m.v)    #$ warning[W2192]: `support.old.SEEN` is deprecated since 0.6: count with `LIMIT`
