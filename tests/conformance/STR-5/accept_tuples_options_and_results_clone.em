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

    fn clear(mut self):
        self.name = None

fn main():
    x: Option[String] = Some("a")
    r: Result[String, int] = Ok("b")
    t = (String.from("c"), 1)
    h = Holder(Some("d"))
    println(x.clone(), r.clone(), t.clone(), h.clone())
    a = Node(name=Some("hello"))
    b = a
    kept = a.name.clone()
    match a.name.clone():
        Some(s):
            b.clear()
            println(s)
        None:
            pass
    println(kept, a.name)
