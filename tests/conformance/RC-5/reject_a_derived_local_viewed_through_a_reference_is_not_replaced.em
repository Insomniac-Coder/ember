#$ test: compile-fail
#$ rules: RC-5, BRW-2
# `[RC-5]` (D-678) — the view an inherited getter returns through a reference borrows the derived
# local, so the local is not written while the view lives.

open class Base:
    items: Array[int]

    fn init(mut self):
        self.items = [1, 2]

    fn view(self) -> Span[int]:
        return self.items[..]

class Derived(Base):
    fn init(mut self):
        super.init()

fn main():
    d = Derived()
    r = ref d
    v = r.view()
    d = Derived()      #$ error[E3021]: `d` cannot be written while it is borrowed
    println(v)
