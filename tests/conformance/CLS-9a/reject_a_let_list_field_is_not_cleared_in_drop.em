#$ test: compile-fail
#$ rules: CLS-9a, DRP-1
# `[CLS-9a]` — after `init`, nothing is changed through a `let` field, `drop` included: a list
# field is not cleared there either (`E2104`).

class Cart:
    let items: Array[int] = []

    fn drop():
        this.items.clear()    #$ error[E2104]: `items` is `let`: it is set in `init` and never changed after; to add to it, write the field without `let`: `items: Array[int]`

fn main():
    _c = Cart()
