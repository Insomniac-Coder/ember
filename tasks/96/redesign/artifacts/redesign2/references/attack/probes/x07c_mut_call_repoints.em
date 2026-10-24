# Attack on A7 x A5: a `mut self` method re-points a view field into the value's own characters.
# Before the call the caller knows `t.cur` views `text`; unless the call's summary says "after me,
# cur may view buf", the caller lets `t.buf = "x"` free what `t.cur` now points into.
struct Tok:
    buf: String
    cur: str

    fn refill(mut self):
        self.buf = "fresh words"
        self.cur = self.buf[0..5]

fn main():
    text: String = "hello"
    t = Tok(buf="", cur=text[0..5])
    t.refill()
    t.buf = "x"
    println(t.cur)
