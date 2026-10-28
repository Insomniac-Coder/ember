#$ test: compile-fail
#$ rules: R12, R4
#$ help: b.on_click(fn() with owns count changes Counter: count.n += 1)
## R12 (rules.md lines 624-627, 651-652): the real variables a lambda captures are checked in the function
## that writes it, which sees every place the function value goes; a function value may not outlive a
## place it uses and does not own (E3026). AN-57's program; rules.md section 3's line 864 message, word
## for word, at the lambda's line.

class Button:
    handlers: Array[fn() changes captured] = []

    fn on_click(owns f: fn() changes captured) changes handlers:
        this.handlers.push(f)

    fn click() changes handlers, captured:
        for ref mut h in this.handlers:
            h()

fn register(ref mut b: Button, ref mut total: int):
    b.on_click(fn() changes total: total += 1)    #$ error[E3026]: the click handler changes `total`, which belongs to `register`'s caller, and `b` keeps the handler after `register` returns; keep the total in a value the handler can own a share of: declare `struct Counter:` with `n: int = 0`; in the caller write `count = Counter()` and call `register(ref mut b, shares mut count)`; write `fn register(ref mut b: Button, owns count: shares mut Counter):` and `b.on_click(fn() with owns count changes Counter: count.n += 1)`; and in `Button` keep such handlers as `handlers: Array[fn() changes Counter]`, with `fn on_click(owns f: fn() changes Counter) changes handlers:` and `fn click() changes handlers, Counter:`

fn main():
    total = 0
    b = Button()
    register(ref mut b, ref mut total)
    b.click()
    println(total)
