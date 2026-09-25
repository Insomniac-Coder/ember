#$ test: compile-fail
#$ rules: STR-5
# D-293 — a type has `Eq` when every part has it. A part with a written `eq`
# next to one that opted out does not make the whole comparable: `==` on an
# `Array` or a tuple of them is `E2040` (the compiler used to panic on the
# array and compare the tuple's other part by bytes).

struct S:
    x: int

extend S implements Eq:
    fn eq(self, other: S) -> bool:
        return self.x == other.x

@no_derive(Eq)
struct Tag:
    n: int

struct W:
    s: S
    t: Tag

fn main():
    a = [W(S(1), Tag(0))]
    b = [W(S(1), Tag(0))]
    println(a == b)    #$ error[E2040]: `Array[W]` does not implement `Eq`, which `==` needs
    println((S(1), Tag(0)) == (S(1), Tag(5)))    #$ error[E2040]: `(S, Tag)` does not implement `Eq`, which `==` needs
    println(S(1) == S(1))
