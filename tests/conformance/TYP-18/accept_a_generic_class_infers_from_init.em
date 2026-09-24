#$ test: run-pass
#$ rules: TYP-18, CLS-1
#$ stdout:
#$ 1 5
#$ 4.0 1
# `[TYP-18]` — a generic class with an `init` takes its type arguments from
# `init`'s parameters; one without takes them from its fields, by name
# (D-237).

class Box2[T]:
    items: Array[T]

    fn init(mut self, first: T):
        self.items = [first]

class Pt[T]:
    x: T
    y: T

b = Box2(5)
println(len(b.items), b.items[0])
p = Pt(1.5, 2.5)
q = Pt(y=3, x=4)
println(p.x + p.y, q.x - q.y)
