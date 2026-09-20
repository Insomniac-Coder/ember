#$ test: compile-fail
#$ rules: TYP-22, OWN-3, DRP-6

interface Render:
    fn render(self) -> i32

class Pixel implements Render:
    value: i32

    fn render(self) -> i32:
        return self.value

fn main():
    pixel = Pixel(42)
    boxed: Box[dyn Render] = Box(pixel)
    println(pixel.value) #$ error[E3040]: `pixel` has been moved out of
    println(boxed.render())
