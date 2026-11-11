#$ test: run-pass
#$ rules: TYP-16, TYP-22, ENM-1, DRP-6, HEAP-1, MONO-1
#$ profiles: debug, development, release
#$ stdout: 42
#$ stdout: 7
#$ assert-c: contains(static const struct em_vt_dyn_6Render em_vt_dyn_6Render_for_6SignalGaf29ae0d2798f0fb5b8d9c352ca3a3a3)
#$ assert-c: contains(em_vt_dyn_6Render_for_6SignalGaf29ae0d2798f0fb5b8d9c352ca3a3a3_drop)
#$ assert-c: contains(em_vt_dyn_6Render_for_6SignalGaf29ae0d2798f0fb5b8d9c352ca3a3a3_slot0)
#$ assert-c: contains(.data = ember_box_new_copy)

interface Render:
    fn render() -> i32

struct Token:
    value: i32

    fn drop():
        println(this.value)

enum Signal[T] implements Render:
    Ready(value: T)

    fn render() -> i32:
        return 42

fn main():
    boxed: owns dyn Render = owns Signal[Token].Ready(Token(7))
    println(boxed.render())
