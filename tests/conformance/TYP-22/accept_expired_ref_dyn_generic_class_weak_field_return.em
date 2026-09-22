#$ test: run-pass
#$ rules: TYP-16, TYP-22, IFC-1, CLS-4, BRW-1, HEAP-3, HEAP-4, HEAP-5, HEAP-6, HEAP-7, WK-11, WK-12, WK-13, TST-26
#$ profiles: debug, release, shipping
#$ stdout: 7
#$ assert-c: contains("em_vt_dyn_Observe_Observer_bool_slot0")
#$ assert-c: contains("ember_weak_retain(")
#$ assert-c: contains("ember_weak_release(")
#$ assert-c: contains("ember_weak_upgrade(")

# A ref dyn Observe call returns a weak field from a generic class without
# extending either the class or payload lifetime. Once the helper exits, the
# returned observer can only take the expired upgrade path.
interface Observe:
    fn observer(self) -> Weak[Shared[Token]]

struct Token:
    value: i32

class Observer[T] implements Observe:
    weak: Weak[Shared[Token]]
    marker: T

    fn observer(self) -> Weak[Shared[Token]]:
        return self.weak

fn inspect(value: ref dyn Observe) -> Weak[Shared[Token]]:
    return value.observer()

fn expired() -> Weak[Shared[Token]]:
    strong = Shared(Token(7))
    observer = Observer[bool](Weak(strong), true)
    return inspect(ref observer)

fn main():
    weak = expired()
    match weak.upgrade():
        Some(owner):
            println(owner.get().value)
        None:
            println(7)
