#$ test: run-pass
#$ rules: TYP-16, TYP-22, IFC-1, CLS-4, DRP-6, HEAP-1, HEAP-3, HEAP-4, HEAP-6, HEAP-7, WK-11, WK-12, WK-13, TST-26
#$ profiles: debug, release, shipping
#$ stdout: 7
#$ stdout: 42
#$ assert-c: contains("ember_weak_retain(")
#$ assert-c: contains("ember_weak_release(")
#$ assert-c: contains("ember_weak_upgrade(")
#$ assert-c: !contains("ember_box_new_copy")

# A generic class can satisfy two independent interfaces inside one dynamic box
# while owning a Weak[Shared[T]] field. Each interface method uses the same
# class payload; the weak observer stays non-owning and the box stays a handle.
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

fn main():
    strong = Shared(Token(7))
    boxed: Box[dyn Read + Write] = Box(Pair[i32](Weak(strong), 0, 42))
    println(boxed.read())
    println(boxed.write())
