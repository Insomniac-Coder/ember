#$ test: run-pass
#$ rules: TYP-16, TYP-22, DRP-6, HEAP-1
#$ profiles: debug, development, release
#$ stdout: 42
#$ stdout: 42
#$ assert-c: contains(static const struct em_vt_dyn_6Render em_vt_dyn_6Render_for_7PayloadG73498f1dbd18d400)

interface Render:
    fn render(self) -> i32

struct Payload[T] implements Render:
    value: T

    fn render(self) -> i32:
        return 42

fn render_it(value: ref dyn Render) -> i32:
    return value.render()

fn main():
    payload = Payload(42)
    println(render_it(ref payload))
    boxed: Box[dyn Render] = Box(Payload(42))
    println(boxed.render())
