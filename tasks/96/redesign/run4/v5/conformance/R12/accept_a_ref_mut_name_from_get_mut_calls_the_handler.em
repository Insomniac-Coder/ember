#$ test: run-pass
#$ rules: R12, R2, R8
#$ stdout: 3
## R8 (f) and R12 (rules.md lines 517-519, 768-770, 776-778): the stored handler changes its own state
## (its own copy of `n`), so it is called through a name that may change it: a `ref mut` name made from
## `get_mut`, or the list by position. "A call through a `ref mut` name of a stored function value uses
## the name only to copy the function's two words out, before the arguments are given", so `ref mut
## this`, which reaches `h`'s place, may be given to it; `h` is not used after the call. AN-61's second NEW
## form, and its first (by position). The nearest allowed program to
## reject_a_plain_name_from_get_mut_calls_the_handler.

class Ui:
    handlers: Array[fn(ref mut Ui)] = []
    clicks: int = 0

    fn fire():
        i = 0
        while i < len(this.handlers):
            h = ref mut this.handlers.get_mut(i).unwrap()
            h(ref mut this)
            i += 1

    fn fire_by_position():
        i = 0
        while i < len(this.handlers):
            this.handlers[i](ref mut this)
            i += 1

fn main():
    ui = Ui()
    n = 0
    step = fn(ref mut u: Ui) copy of n:
        n += 1
        u.clicks += n
    ui.handlers.push(step)
    ui.fire()
    ui.fire_by_position()
    println(ui.clicks)
