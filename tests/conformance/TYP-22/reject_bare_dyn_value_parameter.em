#$ test: compile-fail
#$ rules: TYP-22, BRW-8
## `[TYP-22]` — `dyn I` has no size of its own: taken over, `owns x: dyn I`, it would be passed by
## value, which is `E2020`. A plain or `ref mut` parameter is passed by address (`[BRW-8]`).

interface Drawable:
    fn draw():
        pass

fn take(owns x: dyn Drawable): #$ error[E2020]: unsized interface value cannot be used by value
    pass

