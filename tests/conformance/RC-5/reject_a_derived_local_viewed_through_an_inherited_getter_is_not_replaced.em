#$ test: compile-fail
#$ rules: RC-5, LT-1, CLS-4
# D-460 — the view an inherited getter returns borrows the derived local
# itself, so the local cannot be replaced while the view lives.

open class Base:
    items: Array[int]

    fn view(self) -> Span[int]:
        return self.items[..]

    fn init(mut self):
        self.items = [1, 2]

class Derived(Base):
    fn init(mut self):
        super.init()

fn main():
    d = Derived()
    v = d.view()
    d = Derived()    #$ error[E3021]: `d` cannot be written while it is borrowed
    println(v)
