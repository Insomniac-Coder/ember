#$ test: compile-fail
#$ rules: CLS-9
# D-567: `[CLS-9]`'s own code, `E2104`.

class Ledger:
    let id: int

    fn init(id: int):
        this.id = id

    fn renumber(id: int):
        this.id = id #$ error[E2104]: `id` is `let`: it is set in `init` and never changed after; to change it, write the field without `let`: `id: int`

fn main():
    l = Ledger(7)
    l.renumber(8)
