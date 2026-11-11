#$ test: run-pass
#$ rules: TYP-22, DRP-6, HEAP-1
#$ profiles: debug, development, release
#$ stdout: 42
#$ assert-c: contains(static const struct em_vt_dyn_6Render em_vt_dyn_6Render_for_7Payload)
#$ assert-c: contains(em_7Payload4bump((em_7Payload*)_0);)

interface Render:
    fn render() -> i32
    fn bump()

struct Payload implements Render:
    value: i32

    fn render() -> i32:
        return this.value

    fn bump():
        this.value = this.value + 1

fn main():
    boxed: owns dyn Render = owns Payload(41)
    boxed.bump()
    println(boxed.render())
