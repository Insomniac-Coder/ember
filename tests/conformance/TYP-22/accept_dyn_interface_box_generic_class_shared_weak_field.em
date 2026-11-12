#$ test: run-pass
#$ rules: TYP-16, TYP-22, IFC-1, CLS-4, DRP-6, HEAP-1, HEAP-3, HEAP-4, HEAP-6, HEAP-7, WK-11, WK-12, WK-13, TST-26
#$ profiles: debug, development, release
#$ stdout: 7
#$ assert-c: contains("ember_weak_retain(")
#$ assert-c: contains("ember_weak_release(")
#$ assert-c: contains("ember_weak_upgrade(")
#$ assert-c: !contains("ember_box_new_copy")

# A generic class carrying weakref T remains a class-handle payload when
# erased into Box[dyn Render]. Dynamic dispatch upgrades its weak field through
# the explicit the owner read as its value boundary; the box later uses class field-drop glue.
interface Render:
    fn render() -> i32

struct Token:
    value: i32

[T]
class Observer implements Render:
    weak: weakref Token
    marker: T

    fn render() -> i32:
        if this.weak exists as owner:
            return owner.value
        else:
            return 0

fn main():
    strong = shares mut Token(7)
    boxed: owns dyn Render = owns Observer[bool](weakref strong, true)
    println(boxed.render())
