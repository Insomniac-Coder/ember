#$ test: run-pass
#$ rules: TYP-22, IFC-1, CLS-4, DRP-6, HEAP-1
#$ profiles: debug, release, shipping
#$ stdout: 42
#$ assert-c: contains(em_vt_dyn_Render_Pixel_drop)
#$ assert-c: !contains(ember_box_new_copy)

interface Render:
    fn render(self) -> i32

open class Base:
    value: i32

    fn init(mut self, value: i32):
        self.value = value

class Pixel(Base) implements Render:
    fn init(mut self, value: i32):
        super.init(value)

    fn render(self) -> i32:
        return self.value

fn main():
    boxed: Box[dyn Render] = Box(Pixel(42))
    println(boxed.render())
