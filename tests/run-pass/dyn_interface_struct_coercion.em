#$ test: run-pass
#$ rules: TYP-22, IFC-1, BRW-8
#$ profiles: debug, release, shipping
#$ stdout: 42
#$ assert-c: contains(static const struct em_vt_dyn_Render em_vt_dyn_Render_Pixel)
#$ assert-c: contains(return em_Pixel_render((em_Pixel*)_0);)
#$ assert-c: contains(em_Pixel_bump((em_Pixel*)_0);)
#$ assert-c: !contains(ember_retain)

interface Render:
    fn render(self) -> i32
    fn bump(mut self)

struct Pixel implements Render:
    value: i32

    fn render(self) -> i32:
        return self.value

    fn bump(mut self):
        self.value = self.value + 1

fn render_it(value: ref mut dyn Render) -> i32:
    value.bump()
    return value.render()

fn main():
    pixel = Pixel(41)
    println(render_it(ref mut pixel))
