#$ test: compile-fail
#$ rules: GRM-34, IFC-4
# ODR-042 — a blanket implementation is checked, for each argument it is
# used with, as a written one is: each method has the interface's signature
# and each associated type is given.

interface Tag[T]:
    fn tag(t: T) -> String

struct Label:
    name: String

extend[T: needs Display] Label implements Tag[T]:    #$ error[E2040]: `Label.tag` does not match the signature required by `Tag[i64]`
    fn tag(t: T) -> int:
        return 1

struct Rows:
    data: Array[int]

extend[I: needs Copy] Rows implements Index[I]:    #$ error[E2040]: `Rows` implements `Index[bool]` but does not say what `Output` is
    fn index(i: I) -> ref int:
        return ref this.data[0]

[L: needs Tag[int]]
fn show(l: L) -> int:
    return 0

[R: needs Index[bool, Output = int]]
fn row(r: R) -> int:
    return 0

fn main():
    println(show(Label(name=String.from("n"))))
    r = Rows(data=[1])
    println(row(r))    #$ error[E2040]: `Rows`'s `Output` for `Index[bool]` is not stated, but `R`'s bound needs `i64`
