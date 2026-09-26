#$ test: run-pass
#$ rules: TYP-16, TYP-22, IFC-1, CLS-4, BRW-1, EXC-1, HEAP-3, HEAP-4, HEAP-5, HEAP-6, HEAP-7, WK-11, WK-12, WK-13, TST-26
#$ profiles: debug, release, shipping
#$ stdout: 7
#$ assert-c: contains("em_vt_dyn_Revise_Observer_bool_slot0")
#$ assert-c: contains("ember_object_begin_write")
#$ assert-c: contains("ember_weak_retain(")
#$ assert-c: contains("ember_weak_release(")
#$ assert-c: contains("ember_weak_upgrade(")

# A ref mut dyn Revise carrier borrows a generic class, updates its counter,
# and returns an independent Weak[Shared[T]] copy. The mutable dynamic access
# ends with the call while the caller may still upgrade the returned observer.
interface Revise:
    fn observer(mut self) -> Weak[Shared[Token]]

struct Token:
    value: i32

class Observer[T] implements Revise:
    weak: Weak[Shared[Token]]
    marker: T
    count: i32

    fn observer(mut self) -> Weak[Shared[Token]]:
        self.count = self.count + 1
        return self.weak

fn inspect(value: ref mut dyn Revise) -> Weak[Shared[Token]]:
    return value.observer()

fn main():
    strong = Shared(Token(7))
    observer = Observer[bool](Weak(strong), true, 0)
    weak = inspect(ref mut observer)
    match weak.upgrade():
        Some(owner):
            println(owner.get().value)
        None:
            println(0)
