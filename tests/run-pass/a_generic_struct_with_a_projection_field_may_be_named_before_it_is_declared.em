#$ test: run-pass
#$ rules: TYP-16, IFC-3
#$ stdout: 3
# `Holder[Q]` holds a `Q.State`, a hidden parameter of every instance. `Maker`'s method names
# `Holder[Q]` before `Holder` is declared: the signature's instance had no hidden parameter and
# the body's had one, so the two were different types ("expected `Holder[Q]`, found `Holder[Q]`").
# A generic struct a signature names before its declaration is now collected first.
interface P:
    type State
    fn state() -> State

struct Maker[Q: P]:
    n: int

    fn make(self) -> Holder[Q]:
        return Holder[Q](state=Q.state(), at=self.n)

struct Holder[Q: P]:
    state: Q.State
    at: int

struct One:
    pass

extend One implements P:
    type State = int

    fn state() -> int:
        return 2

fn main():
    m = Maker[One](n=1)
    h = m.make()
    println(h.state + h.at)
