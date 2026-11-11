#$ test: compile-fail
#$ rules: TYP-22, OWN-3, DRP-6

interface Render:
    fn render() -> i32

class Pixel implements Render:
    value: i32

    fn render() -> i32:
        return this.value

fn main():
    pixel = Pixel(42)
    boxed: owns dyn Render = owns pixel
    println(pixel.value) #$ error[E3040]: `pixel` was handed over on line 15 and is used on line 16; to keep it, write `owns copy of pixel`
    println(boxed.render())
