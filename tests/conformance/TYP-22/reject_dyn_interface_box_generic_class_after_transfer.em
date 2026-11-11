#$ test: compile-fail
#$ rules: TYP-16, TYP-22, OWN-3
#$ profiles: debug, development, release

interface Render:
    fn render() -> i32

[T]
class Pixel implements Render:
    value: i32
    marker: T

    fn render() -> i32:
        return this.value

fn main():
    pixel = Pixel[bool](42, false)
    _boxed: owns dyn Render = owns pixel
    println(pixel.value) #$ error[E3040]: `pixel` was handed over on line 18 and is used on line 19; to keep it, write `owns copy of pixel`
