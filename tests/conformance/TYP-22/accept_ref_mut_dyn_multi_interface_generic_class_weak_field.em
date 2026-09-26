#$ test: run-pass
#$ rules: TYP-16, TYP-22, IFC-1, CLS-4, BRW-1, EXC-1, HEAP-3, HEAP-4, HEAP-5, HEAP-6, HEAP-7, WK-11, WK-12, WK-13, TST-26
#$ profiles: debug, release, shipping
#$ stdout: 8
#$ assert-c: contains("ember_object_begin_write")
#$ assert-c-count: contains("ember_weak_retain(") == 1
#$ assert-c-count: contains("ember_weak_release(") == 1
#$ assert-c: contains("ember_weak_upgrade(")

# An explicit ref mut generic-class borrow coerces to ref mut dyn Read + Bump.
# The mut slot updates the original class, then the shared slot observes its
# Weak[Shared[T]] field without creating an owner copy at the coercion boundary.
interface Read:
    fn read(self) -> i32

interface Bump:
    fn bump(mut self) -> i32

struct Token:
    value: i32

class Counter[T] implements Read, Bump:
    weak: Weak[Shared[Token]]
    marker: T
    count: i32

    fn read(self) -> i32:
        match self.weak.upgrade():
            Some(owner):
                return owner.get().value + self.count
            None:
                return self.count

    fn bump(mut self) -> i32:
        self.count = self.count + 1
        return self.count

fn revise(value: ref mut dyn Read + Bump) -> i32:
    value.bump()
    return value.read()

fn main():
    strong = Shared(Token(7))
    counter = Counter[bool](Weak(strong), true, 0)
    println(revise(ref mut counter))
