#$ test: run-pass
#$ rules: TYP-17, TYP-16
#$ profiles: debug, release, shipping
#$ stdout: 4 8 5
# D-379 — an interface's default method may return a generic type holding
# `Self` (`doubled` gives `Doubled[Self]`), and that generic may implement the
# interface in turn. Each instance's default is made when a call names it:
# made eagerly, `Doubled[Up]`'s `doubled` names `Doubled[Doubled[Up]]`, whose
# own names the next, and checking never ended. Unused, `Doubled[Doubled[Up]]`'s
# `doubled` is never made.

interface Counter:
    fn next(mut self) -> int

    fn doubled(owned self) -> Doubled[Self]:
        return Doubled(self)

struct Doubled[C]:
    inner: C

extend[C: Counter] Doubled[C] implements Counter:
    fn next(mut self) -> int:
        return 2 * self.inner.next()

struct Up:
    n: int

extend Up implements Counter:
    fn next(mut self) -> int:
        self.n += 1
        return self.n

fn main():
    d = Up(0).doubled().doubled()
    first = d.next()
    second = d.next()
    u = Up(4)
    println(first, second, u.next())
