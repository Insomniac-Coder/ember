#$ test: run-pass
#$ rules: TYP-16, TYP-22, IFC-1, CLS-2, CLS-4, DRP-6, HEAP-1
#$ profiles: debug, release, shipping
#$ stdout: 42
#$ assert-c: contains(em_vt_dyn_Render_Pixel_bool_drop)
#$ assert-c: !contains(ember_box_new_copy)

interface Render:
    fn render(self) -> i32

open class Base:
    value: i32

    fn init(mut self, value: i32):
        self.value = value

class Pixel[T](Base) implements Render:
    marker: T

    fn init(mut self, value: i32, marker: T):
        super.init(value)
        self.marker = marker

    fn render(self) -> i32:
        return self.value

fn main():
    boxed: Box[dyn Render] = Box(Pixel[bool](42, false))
    println(boxed.render())
