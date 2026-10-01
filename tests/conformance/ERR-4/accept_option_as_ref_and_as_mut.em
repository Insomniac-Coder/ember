#$ test: run-pass
#$ rules: ERR-4, GRM-13, STD-19
#$ stdout:
#$ Some(3) 1
#$ Some(5) true None
# D-407 (6), the owner's decision of 2026-10-01 — `Option.as_ref` and
# `as_mut` are the compiler's: a reference to the payload where the value
# is, whatever its type (`[GRM-13]` binds a `Copy` payload by value), so a
# `peek` over a held `Option[T]` is written in Ember.

struct Peek[T]:
    peeked: Option[T]

extend[T] Peek[T]:
    fn peek(mut self) -> Option[ref T]:
        return self.peeked.as_ref()

fn add_one(mut o: Option[int]):
    match o.as_mut():
        Some(x):
            x += 1
        None:
            pass

fn main():
    p = Peek(peeked = Some(3))
    q = Peek(peeked = Some("s".to_string()))
    n = 0
    match q.peek():
        Some(s):
            n = s.len()
        None:
            pass
    println(p.peek(), n)
    o: Option[int] = Some(4)
    add_one(o)
    none: Option[int] = None
    println(o, o.as_ref().is_some(), none.as_ref())
