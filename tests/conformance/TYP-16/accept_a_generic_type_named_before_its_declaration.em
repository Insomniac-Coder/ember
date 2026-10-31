#$ test: run-pass
#$ rules: TYP-16
#$ stdout: 3 0
#$ stdout: E.One(c=C(w=4))
# Items may appear in any order: a generic struct's field and method may name
# a generic type declared after it, as a plain struct's may (D-279).

[T]
struct A:
    b: Array[B[T]] = []

    fn make(v: T) -> B[T]:
        return B[T](v)

[T]
struct B:
    v: T

enum E[T]:
    One(c: C[T])

[T]
struct C:
    w: T

fn main():
    a = A[int]()
    println(a.make(3).v, a.b.len())
    e = E[int].One(C[int](4))
    println(e)
