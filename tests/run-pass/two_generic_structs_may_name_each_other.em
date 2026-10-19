#$ test: run-pass
#$ rules: TYP-16
#$ stdout: 7 8

## D-729 — two generic structs that name each other: `A[T]`'s method returns a `B[T]`, whose field
## borrows an `A[T]`. Collecting `A` collects `B` on demand from `A`'s method signature, and `B`'s
## field then names `A`; it was `E1010`, "cannot find type `A`", as `A` was registered only once
## its methods were.

struct A[T]:
    n: T

    fn b(mut self) -> B[T]:
        return B[T](a=ref mut self)

struct B[T]:
    a: ref mut A[T]

    fn get(self) -> T:
        return self.a.n

fn main():
    x = A[int](n=7)
    y = x.b()
    first = y.get()
    x.n += 1
    println(first, x.n)
