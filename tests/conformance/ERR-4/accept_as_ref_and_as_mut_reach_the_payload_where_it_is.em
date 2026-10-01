#$ test: run-pass
#$ rules: ERR-4, GRM-13
#$ stdout: 6
#$ stdout: true
# ODR-094 — `as_ref` and `as_mut` give a reference to an `Option`'s payload
# where it is, a number's too, which a pattern binds by value (`[GRM-13]`).

struct Saved:
    next: Option[int]

extend Saved:
    fn peek(self) -> Option[ref int]:
        return self.next.as_ref()

    fn bump(mut self):
        match self.next.as_mut():
            Some(v):
                v += 1
            None:
                pass

fn main():
    s = Saved(next = Some(5))
    s.bump()
    match s.peek():
        Some(v):
            println(v)
        None:
            println("none")
    e = Saved(next = None)
    println(e.peek().is_none())
