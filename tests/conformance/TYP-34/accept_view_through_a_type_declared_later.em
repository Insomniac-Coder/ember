#$ test: run-pass
#$ rules: TYP-34
#$ stdout: hi Some('yo')
# D-300 — whether a type is a view does not depend on declaration order:
# `Outer` holds an `Inner` declared below it, which holds a `str`, so
# `@view` on `Outer` (and on `Wrapped`) is right. It was E2030, judged
# before `Inner` had its fields.

@view
struct Outer:
    inner: Inner

@view
enum Wrapped:
    One(Inner)
    Empty

struct Inner:
    text: str

fn main():
    o = Outer(Inner("hi"))
    w = Wrapped.One(Inner("yo"))
    shown = match w:
        One(i) => Some(i.text)
        Empty => None
    println(o.inner.text, shown)
