#$ test: run-pass
#$ rules: TYP-22, IFC-1
#$ profiles: debug, development, release
#$ stdout: 42
#$ assert-c: contains(static const struct em_vt_dyn_6Render em_vt_dyn_6Render_for_6Signal)
#$ assert-c: contains(return em_6Signal6render(*(em_6Signal*)_0);)

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
