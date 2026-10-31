#$ test: run-pass
#$ rules: TYP-17, GRM-34
#$ profiles: debug
#$ stdout: 6 16
# D-644 — `where T: Show` bounds `T` as `[T: Show]` does, on a function and on an extension alike:
# the body may call what the bound provides.

interface Show:
    fn show() -> int

extend i64 implements Show:
    fn show() -> int:
        return this * 2

[T]
fn twice(x: T) -> int where T: needs Show:
    return x.show()

[T]
struct Cell2:
    v: T

extend[T] Cell2[T] where T: needs Show:
    fn quad() -> int:
        return this.v.show() * 2

fn main():
    println(twice(3), Cell2(4).quad())
