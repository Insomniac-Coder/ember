#$ test: run-pass
#$ rules: TYP-22, IFC-1, BRW-8
#$ profiles: debug, development, release
#$ stdout: 42
#$ assert-c: contains(static const struct em_vt_dyn_6Render em_vt_dyn_6Render_for_5Pixel)
#$ assert-c: contains(return em_5Pixel6render((em_5Pixel*)_0);)
#$ assert-c: contains(em_5Pixel4bump((em_5Pixel*)_0);)
#$ assert-c: !contains(ember_retain)

interface Render:
    fn render() -> i32
    fn bump()

struct Pixel implements Render:
    value: i32

    fn render() -> i32:
        return this.value

    fn bump():
        this.value = this.value + 1

fn render_it(ref mut value: dyn Render) -> i32:
    value.bump()
    return value.render()

fn main():
    pixel = Pixel(41)
    println(render_it(ref mut pixel))
