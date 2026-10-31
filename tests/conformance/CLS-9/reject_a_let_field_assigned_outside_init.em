#$ test: compile-fail
#$ rules: CLS-9
# D-567: `[CLS-9]`'s own code, `E2104`.

class Ledger:
    let id: int

    fn init(id: int):
        this.id = id

    fn renumber(id: int):
        this.id = id #$ error[E2104]: cannot assign to `let` field `Ledger.id` outside its `init`

fn main():
    l = Ledger(7)
    l.renumber(8)
