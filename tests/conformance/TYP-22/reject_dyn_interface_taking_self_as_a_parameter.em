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

fn a(x: ref dyn Same):    #$ error[E2050]: `same` takes `other: Self`
    pass

fn b(x: ref dyn Merge):    #$ error[E2050]: `merge` takes `other: Self`
    pass

fn c(x: ref dyn Add2):
    pass

fn d(x: ref dyn Inside):    #$ error[E2050]: `put` takes `others: Self`
    pass

fn main():
    pass
