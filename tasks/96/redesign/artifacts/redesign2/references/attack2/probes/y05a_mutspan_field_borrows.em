# Attack on A5's inferred summary (answers report 3.3): a result read through a MutSpan field.
# [LT-1a] says a result read through a `ref mut` field borrows `self` as well. A `MutSpan` field is a
# writable view too. If `@borrows(self.buf)` were accepted here, `a` and `b` would be two writable
# references to xs[0] at once. The inference must refuse it the same way.
struct Cursor:
    buf: MutSpan[int]

    @borrows(self.buf)
    fn cur(mut self) -> ref mut int:
        return ref mut self.buf[0]

fn main():
    xs: Array[int] = [1, 2]
    c = Cursor(buf=xs)
    a = c.cur()
    b = c.cur()
    a += 1
    b += 1
    println(xs[0])
