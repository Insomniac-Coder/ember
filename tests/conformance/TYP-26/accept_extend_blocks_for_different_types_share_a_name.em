#$ test: run-pass
#$ rules: TYP-26, IFC-1
#$ stdout: 4 3
# `[TYP-26]` — two functions of one name are allowed in `extend` blocks for different types.

struct A:
    v: int

struct B:
    v: int

extend A:
    fn size() -> int:
        return this.v + 1

extend B:
    fn size() -> int:
        return this.v

fn main():
    println(A(v=3).size(), B(v=3).size())
