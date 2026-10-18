#$ test: run-pass
#$ rules: RC-5, LT-1
#$ stdout: [1, 2]
#$ [1, 2]
# `[RC-5]`, `[LT-1]` (D-678) — an inherited view getter called on `self` in a derived method, or
# through a reference to a derived local, borrows what it is read through, as one on the local does
# (D-460); upcast into a temporary, the view borrowed a handle the statement released (E3060).

open class Base:
    items: Array[int]

    fn init(mut self):
        self.items = [1, 2]

    fn view(self) -> Span[int]:
        return self.items[..]

class Derived(Base):
    fn init(mut self):
        super.init()

    fn both(self) -> Span[int]:
        return self.view()

fn main():
    d = Derived()
    println(d.both())
    r = ref d
    v = r.view()
    println(v)
