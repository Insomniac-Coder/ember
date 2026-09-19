#$ test: run-pass
#$ rules: TYP-22, IFC-1
#$ profiles: debug, release, shipping
#$ stdout: 42
#$ assert-c: contains(static const struct em_vt_dyn_Factory em_vt_dyn_Factory_Pixel)
#$ assert-c: contains(NULL,)

interface Factory:
    fn clone(self) -> Self where Self: Sized:
        return self

    fn draw(self) -> i32

class Pixel implements Factory:
    value: i32

    fn draw(self) -> i32:
        return self.value

fn draw_it(value: ref dyn Factory) -> i32:
    return value.draw()

fn main():
    pixel = Pixel(42)
    println(draw_it(ref pixel))
