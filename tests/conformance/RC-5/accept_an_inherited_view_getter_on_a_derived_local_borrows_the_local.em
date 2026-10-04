#$ test: run-pass
#$ rules: RC-5, LT-1, CLS-4
#$ profiles: debug, release, shipping
#$ stdout: [1, 2]
#$ stdout: 2 3
#$ assert-c: contains("_up = (struct em_obj_Base*)(*(")
# D-460 — `view` is declared on `Base` and returns a view of the receiver's
# field. Called on a local `Derived`, it borrows that local, as a call on a
# `Base` local does: the view can outlive the statement, and the local cannot
# change while it lives ([LT-1]). Before, the call borrowed a copy of the handle
# the statement's end released, which was `E3060`.

open class Base:
    items: Array[int]

    fn view(self) -> Span[int]:
        return self.items[..]

    fn init(mut self):
        self.items = [1, 2]

class Derived(Base):
    extra: int

    fn init(mut self):
        self.extra = 3
        super.init()

fn main():
    d = Derived()
    v = d.view()
    println(v)
    println(v.len(), d.extra)
