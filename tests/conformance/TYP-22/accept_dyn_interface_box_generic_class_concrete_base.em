#$ test: run-pass
#$ rules: TYP-16, TYP-22, IFC-1, CLS-2, CLS-4, DRP-6, HEAP-1
#$ profiles: debug, development, release
#$ stdout: 42
#$ assert-c: contains(em_vt_dyn_6Render_for_5PixelGda5cc12a188c7619b25d920e0657739a_drop)
#$ assert-c: !contains(ember_box_new_copy)

interface Render:
    fn render() -> i32

open class Base:
    value: i32

    fn init(value: i32):
        this.value = value

[T]
class Pixel(Base) implements Render:
    marker: T

    fn init(value: i32, marker: T):
        this.marker = marker
        super.init(value)

    fn render() -> i32:
        return this.value

fn main():
    boxed: Box[dyn Render] = Box(Pixel[bool](42, false))
    println(boxed.render())
