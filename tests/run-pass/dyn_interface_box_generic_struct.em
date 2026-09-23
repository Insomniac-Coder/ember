#$ test: run-pass
#$ rules: TYP-16, TYP-22, DRP-6, HEAP-1
#$ profiles: debug, release, shipping
#$ stdout: 42
#$ stdout: 42
#$ assert-c: contains(static const struct em_vt_dyn_Render em_vt_dyn_Render_Payload_i64)

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
