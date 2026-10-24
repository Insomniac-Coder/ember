# Attack on A5's inferred summary: a result read through a `ref mut` field. If the summary were
# "the result points into what self.buf views" (not self), `a` and `b` would be two writable
# references to xs[0] at once. Today's `@borrows(self.buf)` refuses it ([LT-1a]); the inference must too.
struct Cursor:
    buf: ref mut Array[int]

    @borrows(self.buf)
    fn cur(mut self) -> ref mut int:
        return ref mut self.buf[0]

fn main():
    xs: Array[int] = [1, 2]
    c = Cursor(buf=ref mut xs)
    a = c.cur()
    b = c.cur()
    a += 1
    b += 1
    println(xs[0])
