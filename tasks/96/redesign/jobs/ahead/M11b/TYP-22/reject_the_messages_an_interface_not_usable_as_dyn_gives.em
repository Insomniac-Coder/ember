# Clause TYP-22.2: "Violations are `E2050`, naming the method and giving the line to write"
#$ test: compile-fail
#$ rules: TYP-22, IFC-5
# The spec's two messages, each at the `dyn` use. Where `E2050` stands (the interface's line or each
# `dyn` use; the existing tests do both) is Q4.

interface Visitor:
    fn see(n: int)

interface Inspect:
    [V: needs Visitor]
    fn make() -> V

interface Same:
    fn same(other: Self) -> bool

fn show(i: dyn Inspect):    #$ error[E2050]: `Inspect` cannot be used as `dyn Inspect`: `make` returns its `V`, and a `dyn` value has one entry per method; take the visitor in instead of returning it, `[V: needs Visitor]` above `fn make(ref mut v: V)`
    pass

fn check(s: dyn Same):    #$ error[E2050]: `Same` cannot be used as `dyn Same`: `same` takes `other: Self`, and through a `dyn Same` the kind of `other` is unknown; take `other: dyn Same` and test its kind in the body, `if other is Tag as o:`, or mark `same` with `where Self: needs Sized` and call it on the concrete types
    pass

fn main():
    pass
