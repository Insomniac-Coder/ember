#$ test: run-pass
#$ rules: IFC-4, TYP-19, TYP-17
#$ stdout: [1, 2, 9]
# D-437 (3) — a correct implementation of a generic method that names the
# extension's parameter matches the interface instance (was E2040, "does not
# match the signature required by `Fill[T]`").

interface Fill[T]:
    [I: needs Iterator[Item = T]]
    fn fill(owns it: I, extra: T)

extend[T: needs Copy] Array[T] implements Fill[T]:
    [I: needs Iterator[Item = T]]
    fn fill(owns it: I, extra: T):
        for x in it:
            this.push(owns x)
        this.push(owns extra)

fn main():
    a: Array[int] = []
    a.fill(owns (1..3).iter(), 9)
    println(a)
