#$ test: run-pass
#$ rules: CLS-9, CLS-9a
#$ stdout: 7 3
# `[CLS-9]` — a `let` field is set in `init`; `[CLS-9a]` — one that is not
# `Copy` may still be changed through.

class Ledger:
    let id: int
    let entries: Array[int] = []

    fn init(id: int):
        this.id = id

    fn record(v: int):
        this.entries.push(v)

fn main():
    l = Ledger(7)
    l.record(1)
    l.record(2)
    l.record(3)
    println(l.id, l.entries.len())
