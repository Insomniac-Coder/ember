#$ test: compile-fail
#$ rules: CLS-9a, CLS-9
# `[CLS-9a]` — after `init`, nothing is changed through a `let` field, `drop` included: a number
# field never changes (`E2104`).

class Counter:
    let n: int

    fn init(n: int):
        this.n = n

    fn drop():
        this.n = 0                  #$ error[E2104]: `n` is `let`: it is set in `init` and never changed after; to change it, write the field without `let`: `n: int`

fn main():
    _c = Counter(1)
