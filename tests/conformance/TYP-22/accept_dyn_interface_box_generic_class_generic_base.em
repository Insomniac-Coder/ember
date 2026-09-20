#$ test: run-pass
#$ rules: TYP-16, TYP-22, IFC-1, CLS-2, CLS-4, DRP-6, HEAP-1
#$ profiles: debug, release, shipping
#$ stdout: 42
#$ assert-c: contains(em_vt_dyn_Render_Pixel_bool_drop)
#$ assert-c: !contains(ember_box_new_copy)

interface Render:
    fn render(self) -> i32

open class Base[T]:
    value: i32
    marker: T

    fn init(mut self, value: i32, marker: T):
        self.value = value
        self.marker = marker

class Pixel[T](Base[T]) implements Render:
    fn init(mut self, value: i32, marker: T):
        super.init(value, marker)

    fn render(self) -> i32:
        return self.value

fn main():
    boxed: Box[dyn Render] = Box(Pixel[bool](42, false))
    println(boxed.render())
