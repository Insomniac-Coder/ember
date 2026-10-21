#$ test: compile-fail
#$ rules: UNS-1, IFC-2
## `[UNS-1]` (D-781) — an implementation of a method its interface declares safe is safe: a call
## through the interface (here the `+` operator) needs no `unsafe` block, so the implementation may
## not need one.

struct P:
    x: int

extend P implements Add:  #$ error[E2040]: `P.add` is `unsafe`
    type Output = P
    unsafe fn add(self, rhs: P) -> P:  #$ warning[W3012]
        return P(self.x + rhs.x)

fn main():
    c = P(1) + P(2)
    println(c.x)
