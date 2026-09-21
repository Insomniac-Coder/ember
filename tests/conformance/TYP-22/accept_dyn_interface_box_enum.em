#$ test: run-pass
#$ rules: TYP-22, IFC-1, DRP-6, HEAP-1
#$ profiles: debug, release, shipping
#$ stdout: 42
#$ stdout: 7
#$ assert-c: contains(static const struct em_vt_dyn_Render em_vt_dyn_Render_Signal)
#$ assert-c: contains(em_vt_dyn_Render_Signal_drop)
#$ assert-c: contains(.data = ember_box_new_copy)

interface Render:
    fn render(self) -> i32

struct Token:
    value: i32

    fn drop(mut self):
        println(self.value)

enum Signal implements Render:
    Ready(Token)

    fn render(self) -> i32:
        return 42

fn main():
    boxed: Box[dyn Render] = Box(Signal.Ready(Token(7)))
    println(boxed.render())
