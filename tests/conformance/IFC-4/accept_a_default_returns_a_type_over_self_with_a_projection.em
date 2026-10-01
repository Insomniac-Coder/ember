#$ test: run-pass
#$ rules: IFC-4, STD-19
#$ stdout: 3
# D-442 — a default method returning a struct made over `Self` that holds a
# projection (`saved: Option[I.Item]`): its signature was read before `Self`
# was a parameter, so the hidden `Self.Item` differed from the body's
# ("expected `P[Self]`, found `P[Self]`").

interface Src:
    type Item
    fn get(mut self) -> Option[Item]

    fn wrap(owned self) -> P[Self]:
        return P(self, None)

struct P[I: Src]:
    inner: I
    saved: Option[I.Item]

struct Ints:
    n: int

extend Ints implements Src:
    type Item = int

    fn get(mut self) -> Option[int]:
        return Some(self.n)

fn main():
    w = Ints(n = 3).wrap()
    println(w.inner.n)
