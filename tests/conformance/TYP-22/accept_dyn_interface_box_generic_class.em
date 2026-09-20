#$ test: run-pass
#$ rules: TYP-16, TYP-22, IFC-1, DRP-6, HEAP-1
#$ profiles: debug, release, shipping
#$ stdout: 42
#$ assert-c: contains(em_vt_dyn_Render_Pixel_bool_drop)
#$ assert-c: !contains(ember_box_new_copy)

interface Render:
    fn render(self) -> i32

class Pixel[T] implements Render:
    value: i32
    marker: T

    fn render(self) -> i32:
        return self.value

fn main():
    boxed: Box[dyn Render] = Box(Pixel[bool](42, false))
    println(boxed.render())
