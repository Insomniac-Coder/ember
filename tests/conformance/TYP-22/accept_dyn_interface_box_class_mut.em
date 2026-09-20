#$ test: run-pass
#$ rules: TYP-22, IFC-1, DRP-6, HEAP-1
#$ profiles: debug, release, shipping
#$ stdout: 42
#$ assert-c: contains(em_vt_dyn_Render_Pixel_slot1)
#$ assert-c: !contains(ember_box_new_copy)

interface Render:
    fn render(self) -> i32
    fn bump(mut self)

class Pixel implements Render:
    value: i32

    fn render(self) -> i32:
        return self.value

    fn bump(mut self):
        self.value = self.value + 1

fn main():
    boxed: Box[dyn Render] = Box(Pixel(41))
    boxed.bump()
    println(boxed.render())
