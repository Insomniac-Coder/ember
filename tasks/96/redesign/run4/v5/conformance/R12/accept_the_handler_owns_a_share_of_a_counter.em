#$ test: run-pass
#$ rules: R12, R3, R4, R9
#$ stdout: 1
## R12 (rules.md lines 681-683, 766-767, 785-786), verdict 32: the E3026 message of the nearest refused
## program followed (reject_a_handler_that_outlives_the_callers_total): the handler owns a share of the
## counter (`owns count`, its own state), so it outlives nothing it uses; it changes the `Counter`
## through that owner. `register` only stores the handler; nothing on any first line says what changes.

class Counter:
    n: int = 0

class Button:
    handlers: Array[fn()] = []

    fn on_click(owns f: fn()):
        this.handlers.push(f)

    fn click():
        for ref mut h in this.handlers:
            h()

fn register(ref mut b: Button, owns count: shares mut Counter):
    b.on_click(fn() owns count: count.n += 1)

fn main():
    count = Counter()
    b = Button()
    register(ref mut b, shares mut count)
    b.click()
    println(count.n)
