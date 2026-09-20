#$ test: run-pass
#$ rules: TYP-22, IFC-1, CLS-4, CLS-6, DRP-1, DRP-6, HEAP-1
#$ profiles: debug, release, shipping
#$ stdout: 42
#$ stdout: pixel
#$ stdout: base
#$ assert-c: contains(em_vt_dyn_Render_Pixel_drop)
#$ assert-c: !contains(ember_box_new_copy)

interface Render:
    fn render(self) -> i32

open class Base:
    value: i32

    fn init(mut self, value: i32):
        self.value = value

    fn drop(mut self):
        println("base")

class Pixel(Base) implements Render:
    fn init(mut self, value: i32):
        super.init(value)

    fn render(self) -> i32:
        return self.value

    fn drop(mut self):
        println("pixel")

fn main():
    boxed: Box[dyn Render] = Box(Pixel(42))
    println(boxed.render())
