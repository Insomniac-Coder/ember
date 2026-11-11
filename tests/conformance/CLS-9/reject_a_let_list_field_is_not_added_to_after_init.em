#$ test: compile-fail
#$ rules: CLS-9, CLS-9a
# `[CLS-9]`, `[CLS-9a]` — a `let` field is set in `init` and nothing is changed through it after:
# a list field is never added to (`E2104`). (This test said the opposite, "one that is not `Copy`
# may still be changed through", which was the rule before 0.9.20.)

class Ledger:
    let id: int
    let entries: Array[int] = []

    fn init(id: int):
        this.id = id

    fn record(v: int):
        this.entries.push(owns v)    #$ error[E2104]: `entries` is `let`: it is set in `init` and never changed after; to add to it, write the field without `let`: `entries: Array[int]`

fn main():
    l = Ledger(7)
    l.record(1)
    println(l.id, l.entries.len())
