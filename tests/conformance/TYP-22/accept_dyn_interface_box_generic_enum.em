#$ test: run-pass
#$ rules: TYP-16, TYP-22, ENM-1, DRP-6, HEAP-1, MONO-1
#$ profiles: debug, development, release
#$ stdout: 42
#$ stdout: 7
#$ assert-c: contains(static const struct em_vt_dyn_6Render em_vt_dyn_6Render_for_6SignalGaf29ae0d2798f0fb)
#$ assert-c: contains(em_vt_dyn_6Render_for_6SignalGaf29ae0d2798f0fb_drop)
#$ assert-c: contains(em_vt_dyn_6Render_for_6SignalGaf29ae0d2798f0fb_slot0)
#$ assert-c: contains(.data = ember_box_new_copy)

interface Render:
    fn render(self) -> i32

struct Token:
    value: i32

    fn drop(mut self):
        println(self.value)

enum Signal[T] implements Render:
    Ready(value: T)

    fn render(self) -> i32:
        return 42

fn main():
    boxed: Box[dyn Render] = Box(Signal[Token].Ready(Token(7)))
    println(boxed.render())
