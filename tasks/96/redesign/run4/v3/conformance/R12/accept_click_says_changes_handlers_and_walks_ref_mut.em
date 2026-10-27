#$ test: run-pass
#$ rules: R12, R5
#$ stdout: 1
#$ stdout: 2
## R12 (rules.md lines 641-644): `click` says `changes handlers` and walks with `ref mut h`, so a handler
## may change its own state (its own copy of `n`, kept across the two clicks).

class Button:
    handlers: Array[fn()] = []

    fn on_click(owns f: fn()) changes handlers:
        this.handlers.push(f)

    fn click() changes handlers:
        for ref mut h in this.handlers:
            h()

fn main():
    b = Button()
    n = 0
    tick = fn() with copy of n:
        n += 1
        println(n)
    b.on_click(tick)
    b.click()
    b.click()
