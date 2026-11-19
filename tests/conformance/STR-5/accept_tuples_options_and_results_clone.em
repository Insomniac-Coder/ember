#$ test: run-pass
#$ rules: STR-5, OWN-8, EXP-6
#$ stdout: Some('a') Ok('b') ('c', 1) Holder(tag=Some('d'))
#$ hello
#$ Some('hello') None
# D-303 — a tuple, an `Option` and a `Result` are `Clone` when their parts
# are, and so is a struct holding one. `.clone()` on them was E1010, which
# left no way to match a class field but by moving out of it (E3012).

struct Holder:
    tag: Option[String]

class Node:
    name: Option[String]

    fn clear():
        this.name = None

fn main():
    x: Option[String] = Some("a")
    r: Result[String, int] = Ok("b")
    t = (String.from("c"), 1)
    h = Holder(Some("d"))
    println(copy of x, copy of r, copy of t, copy of h)
    a = Node(name=Some("hello"))
    b = shares mut a
    kept = copy of a.name
    match copy of a.name:
        Some(s):
            b.clear()
            println(s)
        None:
            pass
    println(kept, a.name)
