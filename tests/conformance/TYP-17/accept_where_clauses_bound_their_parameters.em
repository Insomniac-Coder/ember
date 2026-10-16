#$ test: run-pass
#$ rules: TYP-17, GRM-34
#$ profiles: debug
#$ stdout: 6 16
# D-644 — `where T: Show` bounds `T` as `[T: Show]` does, on a function and on an extension alike:
# the body may call what the bound provides.

interface Show:
    fn show(self) -> int

extend i64 implements Show:
    fn show(self) -> int:
        return self * 2

fn twice[T](x: T) -> int where T: Show:
    return x.show()

struct Cell2[T]:
    v: T

extend[T] Cell2[T] where T: Show:
    fn quad(self) -> int:
        return self.v.show() * 2

fn main():
    println(twice(3), Cell2(4).quad())
