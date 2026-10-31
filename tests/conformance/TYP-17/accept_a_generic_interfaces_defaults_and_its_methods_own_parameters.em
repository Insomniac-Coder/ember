#$ test: run-pass
#$ rules: TYP-17
#$ profiles: debug, development
#$ stdout:
#$ 4
#$ true false
#$ true false
#$ true false
# `[TYP-17]` — an interface with type parameters (D-390, each form of which
# failed): its default bodies reach a type that implements an instance of it
# (`again`: "does not define `again`"); a default with type parameters of its
# own is called with them solved (`near`: "cannot take `P` where it expects
# `U`"); a method bounded by the same instance, defaulted (`same`) or written
# by the implementation (`matches`), neither overflows the compiler's stack
# nor is taken for a different signature.

# The default equality method requires Eq even when every call uses int.
interface Pair[T: needs Eq]:
    fn first() -> T
    fn again() -> T:
        x: T = this.first()
        return x
    [U]
    fn near(other: U, answer: bool) -> bool:
        return answer
    [U: needs Pair[T]]
    fn same(other: U) -> bool:
        return this.first() == other.first()
    [U: needs Pair[T]]
    fn matches(other: U) -> bool

struct P:
    a: int

extend P implements Pair[int]:
    fn first() -> int:
        return this.a

    [U: needs Pair[int]]
    fn matches(other: U) -> bool:
        return this.a == other.first()

fn main():
    println(P(4).again())
    word = "x"
    println(f"{P(1).near(P(2), true)} {P(1).near(word, false)}")
    println(f"{P(1).same(P(1))} {P(1).same(P(2))}")
    println(f"{P(3).matches(P(3))} {P(3).matches(P(4))}")
