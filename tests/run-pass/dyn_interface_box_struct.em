#$ test: run-pass
#$ rules: TYP-22, DRP-6, HEAP-1
#$ profiles: debug, development, release
#$ stdout: 42
#$ stdout: 42
#$ assert-c: contains(static const struct em_vt_dyn_6Render em_vt_dyn_6Render_for_7Payload)
#$ assert-c: contains(.data = ember_box_new_copy)
#$ assert-c-order: "em_7Payload4drop(&(*(em_7Payload*)_0));" then "ember_free("

interface Render:
    fn render() -> i32

struct Payload implements Render:
    value: i32

    fn render() -> i32:
        return this.value

    fn drop():
        println(this.value)

fn main():
    boxed: Box[dyn Render] = Box(Payload(42))
    println(boxed.render())
