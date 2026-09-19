#$ test: run-pass
#$ rules: TYP-22, DRP-6, HEAP-1
#$ profiles: debug, release, shipping
#$ stdout: 42
#$ stdout: 42
#$ assert-c: contains(static const struct em_vt_dyn_Render em_vt_dyn_Render_Payload)
#$ assert-c: contains(.data = ember_box_new_copy)
#$ assert-c-order: "em_Payload_drop(&(*(em_Payload*)_0));" then "ember_free("

interface Render:
    fn render(self) -> i32

struct Payload implements Render:
    value: i32

    fn render(self) -> i32:
        return self.value

    fn drop(mut self):
        println(self.value)

fn main():
    boxed: Box[dyn Render] = Box(Payload(42))
    println(boxed.render())
