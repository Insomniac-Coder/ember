# Diagnostic check (second spelling): returning a reborrow of what a `ref mut int` field points to.
struct Inner:
    r: ref mut int

struct Outer:
    inner: Inner

    fn get(mut self) -> ref mut int:
        return self.inner.r

fn main():
    x = 1
    o = Outer(inner=Inner(r=ref mut x))
    a = o.get()
    a += 1
    println(x)
