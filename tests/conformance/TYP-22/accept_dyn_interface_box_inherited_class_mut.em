#$ test: run-pass
#$ rules: TYP-22, IFC-1, CLS-4, CLS-7, DRP-6, HEAP-1
#$ profiles: debug, development, release
#$ stdout: 42
#$ assert-c: contains(em_vt_dyn_6Render_for_5Pixel_slot1)
#$ assert-c: !contains(ember_box_new_copy)

interface Render:
    fn render() -> i32
    fn bump()

open class Base:
    value: i32

    fn init(value: i32):
        this.value = value

class Pixel(Base) implements Render:
    fn init(value: i32):
        super.init(value)

    fn render() -> i32:
        return this.value

    fn bump():
        this.value = this.value + 1

fn main():
    boxed: owns dyn Render = owns Pixel(41)
    boxed.bump()
    println(boxed.render())
