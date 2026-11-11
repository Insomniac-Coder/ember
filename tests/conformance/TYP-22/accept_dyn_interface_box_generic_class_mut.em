#$ test: run-pass
#$ rules: TYP-16, TYP-22, IFC-1, CLS-7, DRP-6, HEAP-1
#$ profiles: debug, development, release
#$ stdout: 42
#$ assert-c: contains(em_vt_dyn_6Render_for_5PixelGda5cc12a188c7619b25d920e0657739a_slot1)
#$ assert-c: !contains(ember_box_new_copy)

interface Render:
    fn render() -> i32
    fn bump()

[T]
class Pixel implements Render:
    value: i32
    marker: T

    fn render() -> i32:
        return this.value

    fn bump():
        this.value = this.value + 1

fn main():
    boxed: Box[dyn Render] = Box(Pixel[bool](41, false))
    boxed.bump()
    println(boxed.render())
