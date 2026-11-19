#$ test: run-pass
#$ rules: CLS-4
#$ stdout: 6
# `[CLS-4]` — a generic base's type parameters stand for the arguments the derived class gives them.

[T]
open class Box:
    virtual fn put(x: T) -> T:
        return x

class Twice(Box[int]):
    override fn put(x: int) -> int:
        return x + x

fn main():
    t = Twice()
    println(t.put(3))
