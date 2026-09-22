#$ test: run-pass
#$ rules: TYP-16, TYP-22, IFC-1, CLS-2, CLS-4, DRP-6, HEAP-1, HEAP-3, HEAP-4, HEAP-6, HEAP-7, WK-11, WK-12, WK-13, TST-26
#$ profiles: debug, release, shipping
#$ stdout: 7
#$ assert-c: contains("ember_weak_retain(")
#$ assert-c: contains("ember_weak_release(")
#$ assert-c: contains("ember_weak_upgrade(")
#$ assert-c: !contains("ember_box_new_copy")

# A generic derived class uses its inherited Weak[Shared[T]] field through a
# Box[dyn Render] adapter. Base layout and super.init retain/drop the weak
# observer normally; dynamic dispatch still observes the payload via get().
interface Render:
    fn render(self) -> i32

struct Token:
    value: i32

open class Base[T]:
    weak: Weak[Shared[Token]]
    marker: T

    fn init(mut self, weak: Weak[Shared[Token]], marker: T):
        self.weak = weak
        self.marker = marker

class Pixel[T](Base[T]) implements Render:
    fn init(mut self, weak: Weak[Shared[Token]], marker: T):
        super.init(weak, marker)

    fn render(self) -> i32:
        match self.weak.upgrade():
            Some(owner):
                return owner.get().value
            None:
                return 0

fn main():
    strong = Shared(Token(7))
    boxed: Box[dyn Render] = Box(Pixel[bool](Weak(strong), true))
    println(boxed.render())
