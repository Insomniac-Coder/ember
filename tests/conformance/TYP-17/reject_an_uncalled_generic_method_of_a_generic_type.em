#$ test: compile-fail
#$ rules: TYP-17, TYP-16
# D-280 — a generic type's methods with type parameters of their own are
# checked once with every parameter opaque, as its other methods are: a
# body may use only what the bounds provide, even if no call reaches it.
# They were skipped, and checked only per concrete owner. `pick`'s instance for `StrView` returns
# a view of `u`, a source there, so its callers view `u` and `this` alike (`[LT-1]`, `[LT-22]`).

[T]
struct Holder:
    v: T

    [U]
    fn pick(u: U) -> U:
        return u

    [U]
    fn broken(u: U) -> int:
        return u.size()    #$ error[E2040]: `U` has no method `size`; its bounds do not provide one

fn main():
    h = Holder[int](1)
    println(h.pick("x"))
