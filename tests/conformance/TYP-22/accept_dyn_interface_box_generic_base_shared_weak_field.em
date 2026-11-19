#$ test: run-pass
#$ rules: TYP-16, TYP-22, IFC-1, CLS-2, CLS-4, DRP-6, HEAP-1, HEAP-3, HEAP-4, HEAP-6, HEAP-7, WK-11, WK-12, WK-13, TST-26
#$ profiles: debug, development, release
#$ stdout: 7
#$ assert-c: contains("ember_weak_retain(")
#$ assert-c: contains("ember_weak_release(")
#$ assert-c: contains("ember_weak_upgrade(")
#$ assert-c: !contains("ember_box_new_copy")

# A generic derived class uses its inherited weakref T field through a
# Box[dyn Render] adapter. Base layout and super.init retain/drop the weak
# observer normally; dynamic dispatch still observes the payload via get().
interface Render:
    fn render() -> i32

struct Token:
    value: i32

[T]
open class Base:
    weak: weakref Token
    marker: T

    fn init(weak: weakref Token, marker: T):
        this.weak = weakref weak
        this.marker = marker

[T]
class Pixel(Base[T]) implements Render:
    fn init(weak: weakref Token, marker: T):
        super.init(weak, marker)

    fn render() -> i32:
        if this.weak exists as owner:
            return owner.value
        else:
            return 0

fn main():
    strong = shares mut Token(7)
    boxed: owns dyn Render = owns Pixel[bool](weakref strong, true)
    println(boxed.render())
