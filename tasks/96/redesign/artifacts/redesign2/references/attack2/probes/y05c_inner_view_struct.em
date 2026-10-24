# Attack on A5 3.3: the `ref mut` sits one level down, inside a view struct field. [LT-1a]'s rule
# ("a result read through a `ref mut` field borrows `self` as well") must apply at any depth.
struct Inner:
    r: ref mut Array[int]

struct Outer:
    inner: Inner

    @borrows(self.inner)
    fn get(mut self) -> ref mut int:
        return ref mut self.inner.r[0]

fn main():
    xs: Array[int] = [1, 2]
    o = Outer(inner=Inner(r=ref mut xs))
    a = o.get()
    b = o.get()
    a += 1
    b += 1
    println(xs[0])
