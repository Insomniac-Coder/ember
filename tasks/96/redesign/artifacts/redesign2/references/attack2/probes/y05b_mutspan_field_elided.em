# The same with nothing written: what elision (today) and inference (proposed) must both give.
struct Cursor:
    buf: MutSpan[int]

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
