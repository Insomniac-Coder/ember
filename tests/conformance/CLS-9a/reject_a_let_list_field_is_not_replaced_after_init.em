#$ test: compile-fail
#$ rules: CLS-9a, CLS-9
# `[CLS-9a]` — after `init`, nothing is changed through a `let` field: the message says the field is
# `let`, that `init` sets it, and gives the line without `let` for a list that is to grow.

class Cart:
    let items: Array[int] = []

    fn clear():
        this.items = []     #$ error[E2104]: `items` is `let`: it is set in `init` and never changed after; to add to it, write the field without `let`: `items: Array[int]`

fn main():
    c = Cart()
    c.clear()
