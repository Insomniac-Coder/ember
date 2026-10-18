#$ test: run-pass
#$ rules: RC-5, LT-1, CLS-4
#$ stdout: [1, 2, 3]
#$ assert-c: contains("_up = (struct em_obj_Base*)(*(")
# `[RC-5]`, `[LT-1]` (D-677) — an inherited generic method that returns a view of its receiver, called
# on a derived local, takes the borrow of the local upcast to the class that declares it, as a
# non-generic one does (D-460). The generic path passed the `ref Derived` unchanged where the
# instance takes a `ref Base`: gcc refused the C ("incompatible pointer type").

open class Base:
    items: Array[int]

    fn init(mut self):
        self.items = [1, 2, 3]

    fn pick[T](self, tag: T) -> Span[int]:
        return self.items[..]

class Derived(Base):
    fn init(mut self):
        super.init()

fn main():
    d = Derived()
    v = d.pick(7)
    println(v)
