# The same with nothing written: today's elision ties the result to all of `c`, so the second call
# is refused while `a` is in use. An inferred summary must keep this, or it hands out two `ref mut`.
struct Cursor:
    buf: ref mut Array[int]

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
