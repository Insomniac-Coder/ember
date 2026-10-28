#$ test: run-pass
#$ rules: BRW-15
#$ stdout: 1
#$ stdout: 2
## design rules: R12, R5
## R12 (rules.md lines 776-779): `click` walks with `ref mut h`, a name that may change the handler, so a
## handler may change its own state (its own copy of `n`, kept across the two clicks); nothing on `click`'s
## first line says so (verdict 86). The nearest allowed program to
## reject_click_walks_stateful_handlers_with_a_plain_name.

class Button:
    handlers: Array[fn()] = []

    fn on_click(owns f: fn()):
        this.handlers.push(f)

    fn click():
        for ref mut h in this.handlers:
            h()

fn main():
    b = Button()
    n = 0
    tick = fn() copy of n:
        n += 1
        println(n)
    b.on_click(tick)
    b.click()
    b.click()
