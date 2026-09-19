#$ test: run-pass
#$ rules: TYP-22, IFC-1
#$ profiles: debug, release, shipping
#$ stdout: 42
#$ assert-c: contains(static const struct em_vt_dyn_Render em_vt_dyn_Render_Pixel)
#$ assert-c: contains(return em_Pixel_render((struct em_obj_Pixel*)_0);)
#$ assert-c: !contains(ember_retain)

interface Render:
    fn render(self) -> i32

class Pixel implements Render:
    value: i32

    fn render(self) -> i32:
        return self.value

fn render_it(value: ref dyn Render) -> i32:
    return value.render()

fn main():
    pixel = Pixel(42)
    println(render_it(ref pixel))
