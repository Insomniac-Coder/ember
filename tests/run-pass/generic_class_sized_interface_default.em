#$ test: run-pass
#$ rules: TYP-16, TYP-22, IFC-1, DRP-6, HEAP-1
#$ profiles: debug, release, shipping
#$ stdout: 42
#$ stdout: 42
#$ stdout: 42
#$ assert-c: contains(static const struct em_vt_dyn_Factory em_vt_dyn_Factory_Pixel_bool)
#$ assert-c: contains(NULL,)

interface Factory:
    fn clone(self) -> Self where Self: Sized:
        return self

    fn draw(self) -> i32

class Pixel[T] implements Factory:
    value: i32
    marker: T

    fn draw(self) -> i32:
        return self.value

fn draw_it(value: ref dyn Factory) -> i32:
    return value.draw()

fn main():
    pixel = Pixel[bool](42, false)
    clone = pixel.clone()
    println(clone.draw())
    println(draw_it(ref pixel))
    boxed: Box[dyn Factory] = Box(Pixel[bool](42, false))
    println(boxed.draw())
