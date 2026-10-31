#$ test: run-pass
#$ rules: OBJ-2, DSP-3, RC-1, IFC-1
#$ profiles: debug, development, release
#$ stdout: 42
#$ assert-c: contains(static const ember_itable_entry em_itables_5Pixel[])
#$ assert-c: contains(ember_itable_lookup)

# `[OBJ-2]` keeps this erased class handle one pointer wide. `[DSP-3]` then
# obtains Render's concrete implementation from Pixel's type information.
interface Render:
    fn render() -> i32

class Pixel implements Render:
    value: i32

    fn render() -> i32:
        return this.value

fn draw(value: Render) -> i32:
    return value.render()

fn main():
    pixel = Pixel(42)
    println(draw(pixel))
