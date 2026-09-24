#$ test: compile-fail
#$ rules: TYP-16, TYP-22, OWN-3
#$ profiles: debug, release, shipping

interface Render:
    fn render(self) -> i32

class Pixel[T] implements Render:
    value: i32
    marker: T

    fn render(self) -> i32:
        return self.value

fn main():
    pixel = Pixel[bool](42, false)
    _boxed: Box[dyn Render] = Box(pixel)
    println(pixel.value) #$ error[E3040]: `pixel` has been moved out of
