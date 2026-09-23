#$ test: run-pass
#$ rules: ERR-4
#$ profiles: debug, release, shipping
#$ stdout: first
#$ fallback
#$ third
#$ 7
# `unwrap`, `unwrap_or` and `expect` on an `Option`, with an owned payload: the
# value moves out, and an unused default is dropped.

fn main():
    a: Option[String] = Some("first")
    println(a.unwrap())
    b: Option[String] = None
    println(b.unwrap_or("fallback"))
    c: Option[String] = Some("third")
    println(c.unwrap_or("unused"))
    d: Option[int] = Some(7)
    println(d.expect("d holds a number"))
