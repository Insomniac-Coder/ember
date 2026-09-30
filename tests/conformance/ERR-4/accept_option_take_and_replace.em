#$ test: run-pass
#$ rules: ERR-4, OWN-6
#$ stdout: Some(5) None
#$ stdout: None Some(7)
#$ stdout: Some('a') None
# `[ERR-4]` — `Option`'s `take` gives the payload and leaves `None`, and
# `replace` gives it and leaves `Some(value)`; the payload is moved, not
# copied. Neither existed.

fn main():
    o: Option[int] = Some(5)
    t = o.take()
    println(t, o)
    r = o.replace(7)
    println(r, o)
    s: Option[String] = Some(String.from("a"))
    u = s.take()
    println(u, s)
