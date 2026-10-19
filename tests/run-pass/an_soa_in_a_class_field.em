#$ test: run-pass
#$ rules: SOA-2
#$ stdout: 1 3.0
#$ 6.0
# An `SoA` may be a class's field: its helpers come before the class's drop glue, which drops it
# (the C failed: a helper was used before it was declared), and a column of it reads the length
# before it borrows the column, so the run-time access check sees no overlap (it panicked).
open class AnyStorage:
    len: int

    fn init(mut self):
        self.len = 0

    virtual fn remove_at(mut self, i: int):
        pass

class Storage[T](AnyStorage):
    dense: Array[T]
    columns: SoA[T]
    soa: bool

    fn init(mut self, soa: bool):
        self.dense = []
        self.columns = SoA[T].with_capacity(0)
        self.soa = soa
        super.init()

    fn add(mut self, owned value: T):
        if self.soa:
            self.columns.push(value)
        else:
            self.dense.push(value)
        self.len += 1

    override fn remove_at(mut self, i: int):
        if self.soa:
            _gone = self.columns.swap_remove(i)
        else:
            _gone2 = self.dense.swap_remove(i)
        self.len -= 1

@derive(Copy)
struct Pos:
    x: f32
    y: f32

fn main():
    s = Storage[Pos](true)
    s.add(Pos(x=1.0, y=2.0))
    s.add(Pos(x=3.0, y=4.0))
    a: AnyStorage = s
    a.remove_at(0)
    println(s.len, s.columns.x[0])
    t = Storage[Pos](false)
    t.add(Pos(x=5.0, y=6.0))
    println(t.dense[0].y)
