#$ test: run-pass
#$ rules: TYP-22, DRP-6, HEAP-1
#$ profiles: debug, release, shipping
#$ stdout: 42
#$ assert-c: contains(static const struct em_vt_dyn_Render em_vt_dyn_Render_Payload)
#$ assert-c: contains(em_Payload_bump((em_Payload*)_0);)

interface Render:
    fn render(self) -> i32
    fn bump(mut self)

struct Payload implements Render:
    value: i32

    fn render(self) -> i32:
        return self.value

    fn bump(mut self):
        self.value = self.value + 1

fn main():
    boxed: Box[dyn Render] = Box(Payload(41))
    boxed.bump()
    println(boxed.render())
