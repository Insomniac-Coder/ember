#$ test: run-pass
#$ rules: GRM-34, TYP-17
#$ profiles: debug, release, shipping
#$ stdout: 5
#$ 91
#$ 2
# D-605 — a blanket implementation whose interface's member is an associated
# function (no `self`): `B` converts from every `T: Named`, and the function
# is the type's, generic over `T`, as a blanket method is.

interface Conv[T]:
    fn conv(owned value: T) -> Self

interface Named:
    fn name(self) -> int

struct A implements Named:
    v: int

    fn name(self) -> int:
        return self.v

struct C implements Named:
    w: int

    fn name(self) -> int:
        return self.w * 10

struct B:
    v: int

extend[T: Named] B implements Conv[T]:
    fn conv(owned value: T) -> B:
        return B(value.name() + 1)

fn up[E, F: Conv[E]](owned e: E) -> F:
    return F.conv(e)

fn main():
    println(B.conv(A(4)).v)
    println(B.conv(C(9)).v)
    d: B = up(A(1))
    println(d.v)
