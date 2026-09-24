#$ test: run-pass
#$ rules: CLS-4, DSP-1
#$ stdout: 1 2 3 5 2 p
# `[CLS-4]` (ODR-028) — an `override` is virtual in turn, so a further
# subclass may override it again; a call through the base picks the most
# derived method (D-239: the grandchild's `override` was refused). With no
# `init` anywhere in the chain, a derived class gets its base's memberwise
# constructor (`[CLS-10]`).

open class A:
    virtual fn speak(self) -> int:
        return 1

open class B(A):
    override fn speak(self) -> int:
        return 2

class C(B):
    override fn speak(self) -> int:
        return 3

open class P:
    x: int
    name: String = "p"

class Q(P):
    y: int = 2

fn show(a: A) -> int:
    return a.speak()

fn main():
    q = Q(5)
    println(show(A()), show(B()), show(C()), q.x, q.y, q.name)
