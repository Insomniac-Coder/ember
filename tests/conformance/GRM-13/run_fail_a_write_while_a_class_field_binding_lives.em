#$ test: run-fail
#$ rules: GRM-13, EXC-18, EXC-1
#$ profiles: debug, release
#$ panics: exclusivity violation: write access to Bag.name while a read access to Bag.name is active
# `[EXC-18]` — a `match` binding into a class field holds the field's read
# access while it lives: clearing the field through another handle meanwhile
# panics, where it would free the `String` the binding points at (D-302's
# use-after-free, now caught at run time; D-459).

class Bag:
    name: Option[String]

    fn init(mut self):
        self.name = Some("ann")

    fn clear_name(self):
        self.name = None

fn main():
    b = Bag()
    c = b
    match b.name:
        Some(s):
            c.clear_name()
            println(s)
        None:
            pass
