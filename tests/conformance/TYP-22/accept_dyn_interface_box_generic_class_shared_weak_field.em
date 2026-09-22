#$ test: run-pass
#$ rules: TYP-16, TYP-22, IFC-1, CLS-4, DRP-6, HEAP-1, HEAP-3, HEAP-4, HEAP-6, HEAP-7, WK-11, WK-12, WK-13, TST-26
#$ profiles: debug, release, shipping
#$ stdout: 7
#$ assert-c: contains("ember_weak_retain(")
#$ assert-c: contains("ember_weak_release(")
#$ assert-c: contains("ember_weak_upgrade(")
#$ assert-c: !contains("ember_box_new_copy")

# A generic class carrying Weak[Shared[T]] remains a class-handle payload when
# erased into Box[dyn Render]. Dynamic dispatch upgrades its weak field through
# the explicit Shared.get() boundary; the box later uses class field-drop glue.
interface Render:
    fn render(self) -> i32

struct Token:
    value: i32

class Observer[T] implements Render:
    weak: Weak[Shared[Token]]
    marker: T

    fn render(self) -> i32:
        match self.weak.upgrade():
            Some(owner):
                return owner.get().value
            None:
                return 0

fn main():
    strong = Shared(Token(7))
    boxed: Box[dyn Render] = Box(Observer[bool](Weak(strong), true))
    println(boxed.render())
