#$ test: run-pass
#$ rules: CLS-9, CLS-9a
#$ stdout: 7 3
# `[CLS-9]`, `[CLS-9a]` — a `let` field is set in `init` (a list in it filled there too) and only
# read after; a list that is to change is written without `let`.

class Ledger:
    let id: int
    entries: Array[int] = []

    fn init(id: int):
        this.id = id

    fn record(v: int):
        this.entries.push(owns v)

fn main():
    l = Ledger(7)
    l.record(1)
    l.record(2)
    l.record(3)
    println(l.id, l.entries.len())
