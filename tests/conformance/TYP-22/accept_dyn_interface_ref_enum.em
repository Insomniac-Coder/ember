#$ test: run-pass
#$ rules: TYP-22, IFC-1
#$ profiles: debug, release, shipping
#$ stdout: 42
#$ assert-c: contains(static const struct em_vt_dyn_Render em_vt_dyn_Render_Signal)
#$ assert-c: contains(return em_Signal_render(*(em_Signal*)_0);)

interface Render:
    fn render(self) -> i32

enum Signal implements Render:
    Ready

    fn render(self) -> i32:
        return 42

fn render_it(value: ref dyn Render) -> i32:
    return value.render()

fn main():
    signal = Signal.Ready
    println(render_it(ref signal))
