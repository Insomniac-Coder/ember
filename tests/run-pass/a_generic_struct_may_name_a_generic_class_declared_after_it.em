#$ test: run-pass
#$ stdout: 3
#$ 5
# A generic struct's field may name a generic class declared after it, as it may a generic
# struct: the class is collected when the struct's field first names it (it was "cannot find
# type" before). A struct holding a class handle reaches the class's fields through it.
struct Holder[T]:
    store: Store[T]
    at: int

class Store[T]:
    items: Array[T]

    fn init(mut self):
        self.items = []

struct Pos:
    x: int

fn bump(h: Holder[Pos]):
    h.store.items[h.at].x += 2

fn main():
    s = Store[Pos]()
    s.items.push(Pos(x=3))
    h = Holder[Pos](store=s, at=0)
    println(s.items[0].x)
    bump(h)
    println(s.items[0].x)
