#$ test: run-pass
#$ rules: TYP-16, TYP-22, IFC-1, CLS-4, HEAP-3, HEAP-4, HEAP-6, HEAP-7, WK-11, WK-12, WK-13, TST-26
#$ profiles: debug, release, shipping
#$ stdout: 49
#$ assert-c-count: contains("ember_weak_retain(") == 1
#$ assert-c-count: contains("ember_weak_release(") == 1
#$ assert-c: contains("ember_weak_upgrade(")

# Borrowing a generic class as ref dyn Read + Write uses both concrete adapter
# slots without making an owner copy. Its Weak[Shared[T]] field remains owned
# only by the class object throughout the borrowed dynamic call.
# `[TYP-17]` — `write` returns an `i32` field, not the `T` one: a generic
# class's method must hold for every `T` (D-248).
interface Read:
    fn read(self) -> i32

interface Write:
    fn write(self) -> i32

struct Token:
    value: i32

class Pair[T] implements Read, Write:
    weak: Weak[Shared[Token]]
    marker: T
    score: i32

    fn read(self) -> i32:
        match self.weak.upgrade():
            Some(owner):
                return owner.get().value
            None:
                return 0

    fn write(self) -> i32:
        return self.score

fn total(value: ref dyn Read + Write) -> i32:
    return value.read() + value.write()

fn main():
    strong = Shared(Token(7))
    pair = Pair[i32](Weak(strong), 0, 42)
    println(total(ref pair))
