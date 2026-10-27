#$ test: compile-fail
#$ rules: R12, R4
#$ help: b.on_click(fn() with owns count changes Counter: count.n += 1)
## R12 (rules.md lines 741-744, 776-777): the real variables a lambda captures are checked in the function
## that writes it, which sees every place the function value goes; a function value may not outlive a
## place it uses and does not own (E3026). AN-57's program; rules.md section 3's line 1044 message, word
## for word (corrected in v4 to name every line it replaces, `main`'s first line included), at the
## lambda's line. `main` runs the handler through `b.click()` and did not write it, so it says `captured`
## (rules.md lines 680-685).

class Button:
    handlers: Array[fn() changes captured] = []

    fn on_click(owns f: fn() changes captured) changes handlers:
        this.handlers.push(f)

    fn click() changes handlers, captured:
        for ref mut h in this.handlers:
            h()

fn register(ref mut b: Button, ref mut total: int):
    b.on_click(fn() changes total: total += 1)    #$ error[E3026]: the click handler changes `total`, which belongs to `register`'s caller, and `b` keeps the handler after `register` returns; keep the total in an object the handler can own a share of: declare `class Counter:` with the field `n: int = 0`; in `main`, instead of `total = 0`, `register(ref mut b, ref mut total)` and `println(total)`, write `count = Counter()`, `register(ref mut b, shares mut count)` and `println(count.n)`, and make its first line `fn main() changes Counter:`; instead of `register`'s two lines write `fn register(ref mut b: Button, owns count: shares mut Counter):` and `b.on_click(fn() with owns count changes Counter: count.n += 1)`; and in `Button`, instead of its `handlers`, `on_click` and `click` lines, write `handlers: Array[fn() changes Counter] = []`, `fn on_click(owns f: fn() changes Counter) changes handlers:` and `fn click() changes handlers, Counter:`

fn main() changes captured:
    total = 0
    b = Button()
    register(ref mut b, ref mut total)
    b.click()
    println(total)
