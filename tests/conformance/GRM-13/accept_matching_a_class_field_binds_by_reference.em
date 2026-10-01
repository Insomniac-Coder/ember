#$ test: run-pass
#$ rules: GRM-13, EXC-18
#$ profiles: debug, release
#$ stdout: ann
#$ stdout: Some('ann') 3
# `[GRM-13]` — matching a place that is not consumed binds its non-`Copy`
# parts by reference, a class field too: `s` borrows `b.name`, which keeps
# its value, and the loan holds the field's read access while `s` lives
# (D-459; it was a move, `E3012`, until class fields had per-field access).

class Bag:
    name: Option[String]
    items: Array[int]

    fn init(mut self):
        self.name = Some("ann")
        self.items = [1, 2]

    fn grow(self):
        self.items.push(3)

fn main():
    b = Bag()
    c = b
    match b.name:
        Some(s):
            c.grow()
            println(s)
        None:
            pass
    println(b.name, len(b.items))
