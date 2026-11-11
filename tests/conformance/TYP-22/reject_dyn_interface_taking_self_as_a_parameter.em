#$ test: compile-fail
#$ rules: TYP-22
# `[TYP-22]` — an interface that names `Self` as a parameter's type, or inside one, is not usable as
# `dyn`; a method marked `where Self: needs Sized` is excepted.

interface Same:
    fn same(other: Self) -> bool

interface Merge:
    fn merge(owns other: Self)

interface Add2:
    fn add(rhs: Self) -> int where Self: needs Sized:
        todo()

interface Inside:
    fn put(others: Array[Self])

fn a(x: ref dyn Same):    #$ error[E2050]: `Same` cannot be used as `dyn Same`: `same` takes `other: Self`, and through a `dyn Same` the kind of `other` is unknown; take `other: dyn Same` and test its kind in the body
    pass

fn b(x: ref dyn Merge):    #$ error[E2050]: `Merge` cannot be used as `dyn Merge`: `merge` takes `other: Self`, and through a `dyn Merge` the kind of `other` is unknown
    pass

fn c(x: ref dyn Add2):
    pass

fn d(x: ref dyn Inside):    #$ error[E2050]: `Inside` cannot be used as `dyn Inside`: `put` takes `others: Self`, and through a `dyn Inside` the kind of `others` is unknown
    pass

fn main():
    pass
