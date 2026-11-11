#$ test: run-pass
#$ rules: TYP-22, IFC-1, DRP-6, HEAP-1
#$ profiles: debug, development, release
#$ stdout: 42
#$ stdout: 7
#$ assert-c: contains(static const struct em_vt_dyn_6Render em_vt_dyn_6Render_for_6Signal)
#$ assert-c: contains(em_vt_dyn_6Render_for_6Signal_drop)
#$ assert-c: contains(.data = ember_box_new_copy)

interface Render:
    fn render() -> i32

struct Token:
    value: i32

    fn drop():
        println(this.value)

enum Signal implements Render:
    Ready(Token)

    fn render() -> i32:
        return 42

fn main():
    boxed: owns dyn Render = owns Signal.Ready(Token(7))
    println(boxed.render())
