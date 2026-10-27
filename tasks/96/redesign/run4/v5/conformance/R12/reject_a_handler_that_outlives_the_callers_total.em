#$ test: compile-fail
#$ rules: R12, R4
#$ help: b.on_click(fn() owns count: count.n += 1)
## R12 (rules.md lines 752-759, 785-786): "The real variables a lambda captures ... are checked in the
## function that writes the lambda, which sees every place the function value goes: the value may not
## outlive them (E3026)". AN-57's program; rules.md section 3's line 1095 first message, word for word,
## at the lambda's line. The nearest allowed program is accept_the_handler_owns_a_share_of_a_counter.

class Button:
    handlers: Array[fn()] = []

    fn on_click(owns f: fn()):
        this.handlers.push(f)

    fn click():
        for ref mut h in this.handlers:
            h()

fn register(ref mut b: Button, ref mut total: int):
    b.on_click(fn(): total += 1)    #$ error[E3026]: the click handler changes `total`, which belongs to `register`'s caller, and `b` keeps the handler after `register` returns; keep the total in an object the handler can own a share of: declare `class Counter:` with the field `n: int = 0`; in `main`, instead of `total = 0`, `register(ref mut b, ref mut total)` and `println(total)`, write `count = Counter()`, `register(ref mut b, shares mut count)` and `println(count.n)`; and instead of `register`'s two lines write `fn register(ref mut b: Button, owns count: shares mut Counter):` and `b.on_click(fn() owns count: count.n += 1)`

fn main():
    total = 0
    b = Button()
    register(ref mut b, ref mut total)
    b.click()
    println(total)
