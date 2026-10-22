#$ test: compile-fail
#$ rules: TYP-22, TYP-36, STR-5
# `[TYP-22]`, IX.1 (ODR-241) — a `Box[dyn I]` compares only when `I` requires `Eq`, a `Box[T]` only
# when `T` is `Eq` (`E2040`), and a type that is not `Eq` cannot implement an interface requiring
# it.

interface Plain:
    fn size(self) -> int

interface Compared: Eq:
    fn size(self) -> int

@derive(Eq)
struct Block:
    n: int

extend Block implements Plain:
    fn size(self) -> int:
        return self.n

@no_derive(Eq)
struct Opaque:
    n: int

extend Opaque implements Compared:  #$ error[E2040]: `Compared` requires `Eq`, which `Opaque` does not implement
    fn size(self) -> int:
        return self.n

fn main():
    a: Box[dyn Plain] = Box(Block(1))
    b: Box[dyn Plain] = Box(Block(1))
    println(a == b)  #$ error[E2040]: `Box[dyn Plain]` does not implement `Eq`
    println(Box(Opaque(1)) == Box(Opaque(1)))  #$ error[E2040]: `Opaque` does not implement `Eq`
