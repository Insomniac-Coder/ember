#$ test: run-pass
#$ rules: STD-19, ERR-4
#$ stdout: peek 5
#$ stdout: Some(5) Some(6)
#$ stdout: Some(107) None
#$ stdout: Some(1) Some(2) None Some(5)
#$ stdout: Some(4) Some(4) None Some(9)
#$ stdout: peek a
#$ stdout: Some('a') Some('b') None
# ODR-094 — `peekable()`: `peek` and `peek_mut` take the next item early and
# keep it where it is until `next` gives it (numbers too, through `Option`'s
# `as_ref` and `as_mut`); `next_if` and `next_if_eq` take it only when it fits.

fn main():
    xs = [5, 6, 7]
    p = xs.iter().copied().peekable()
    match p.peek():
        Some(v):
            println("peek", v)
        None:
            println("none")
    println(p.next(), p.next())
    match p.peek_mut():
        Some(v):
            v += 100
        None:
            pass
    println(p.next(), p.next())
    ys = [1, 2, 5, 6]
    q = ys.iter().copied().peekable()
    println(q.next_if(fn(x: ref int) => x < 3), q.next_if(fn(x: ref int) => x < 3), q.next_if(fn(x: ref int) => x < 3), q.next())
    zs = [4, 4, 9]
    r = zs.iter().copied().peekable()
    println(r.next_if_eq(4), r.next_if_eq(4), r.next_if_eq(4), r.next())
    words = ["a".to_string(), "b".to_string()]
    w = words.iter().cloned().peekable()
    match w.peek():
        Some(s):
            println("peek", s)
        None:
            pass
    println(w.next(), w.next(), w.next())
