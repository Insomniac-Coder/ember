#$ test: compile-fail
#$ rules: TYP-17, TYP-16
# D-280 — a generic type's methods with type parameters of their own are
# checked once with every parameter opaque, as its other methods are: a
# body may use only what the bounds provide, even if no call reaches it.
# They were skipped, and checked only per concrete owner.

struct Holder[T]:
    v: T

    fn pick[U](self, u: U) -> U:
        return u

    fn broken[U](self, u: U) -> int:
        return u.size()    #$ error[E2040]: `U` has no method `size`; its bounds do not provide one

fn main():
    h = Holder[int](1)
    println(h.pick("x"))
