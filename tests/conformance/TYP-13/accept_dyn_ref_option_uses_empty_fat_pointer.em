#$ test: run-pass
#$ rules: TYP-13, TYP-22
#$ profiles: debug, release, shipping
#$ stdout: 16 16
#$ stdout: 42
#$ stdout: none
# A ref dyn is a non-null data/vtable pair; None uses null data.
interface Render:
    fn render(self) -> i32

class Pixel implements Render:
    value: i32

    fn render(self) -> i32:
        return self.value

fn main():
    pixel = Pixel(42)
    view: ref dyn Render = ref pixel
    println(mem.size_of[Option[ref dyn Render]](), mem.size_of[ref dyn Render]())
    present: Option[ref dyn Render] = Some(view)
    match present:
        Some(item):
            println(item.render())
        None:
            println("unexpected")
    absent: Option[ref dyn Render] = None
    match absent:
        Some(_):
            println("unexpected")
        None:
            println("none")
