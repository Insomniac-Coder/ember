#$ test: run-pass
#$ rules: GRM-13, EXC-18
#$ profiles: debug, development
#$ stdout: ann
#$ stdout: Some('ann') 3
# `[GRM-13]` — matching a place that is not consumed binds its non-`Copy`
# parts by reference, a class field too: `s` borrows `b.name`, which keeps
# its value, and the loan holds the field's read access while `s` lives
# (D-459; it was a move, `E3012`, until class fields had per-field access), so another field of the
# same object may be changed meanwhile (`c.items.push(3)`). (A method that changes its object, as
# `grow` does, holds every field for its call until M6b, and would conflict.)

class Bag:
    name: Option[String]
    items: Array[int]

    fn init():
        this.name = Some("ann")
        this.items = [1, 2]

    fn grow():
        this.items.push(3)

fn main():
    b = Bag()
    c = b
    match b.name:
        Some(s):
            c.items.push(3)
            println(s)
        None:
            pass
    println(b.name, len(b.items))
