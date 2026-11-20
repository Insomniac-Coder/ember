# Clause CLO-4.1: "The real variables a lambda uses are checked in the function that writes it, which
# sees every place the function value goes: the value must provably end before them (...), or it is
# `E3026`, whose message names a word for the place"; "Its message for a handler kept after its maker
# returns"
#$ test: compile-fail
#$ rules: CLO-4, CLO-2, EFF-24
# The spec's message. (`make_counter`'s `E3026` is in `ahead/M6a/DIA-10`.)

class Button:
    handlers: Array[fn()] = []

    fn on_click(owns f: fn()):
        this.handlers.push(owns f)

    fn click():
        for ref mut h in this.handlers:
            h()

fn register(ref mut b: Button, ref mut total: int):
    b.on_click(owns fn(): total += 1)    #$ error[E3026]: the click handler changes `total`, which belongs to `register`'s caller, and `b` keeps the handler after `register` returns; keep the total in an object the handler can own a share of: declare `class Counter:` with the field `n: int = 0`; in `main`, instead of `total = 0`, `register(ref mut b, ref mut total)` and `println(total)`, write `count = Counter()`, `register(ref mut b, shares mut count)` and `println(count.n)`; and instead of `register`'s two lines write `fn register(ref mut b: Button, owns count: shares mut Counter):` and `b.on_click(owns fn() owns count: count.n += 1)`

fn main():
    b = Button()
    total = 0
    register(ref mut b, ref mut total)
    b.click()
    println(total)
