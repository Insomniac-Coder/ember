#$ test: run-pass
#$ rules: OBJ-2, DSP-3, RC-1, IFC-1
#$ profiles: debug, release, shipping
#$ stdout: 42
#$ assert-c: contains(static const ember_itable_entry em_itables_Pixel[])
#$ assert-c: contains(ember_itable_lookup)

# `[OBJ-2]` keeps this erased class handle one pointer wide. `[DSP-3]` then
# obtains Render's concrete implementation from Pixel's type information.
interface Render:
    fn render(self) -> i32

class Pixel implements Render:
    value: i32

    fn render(self) -> i32:
        return self.value

fn draw(value: Render) -> i32:
    return value.render()

fn main():
    pixel = Pixel(42)
    println(draw(pixel))
