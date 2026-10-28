#$ test: run-pass
#$ rules: R12, R5
#$ stdout: 1
## R12 (rules.md lines 564-569), R5: `press` runs the handlers through `b.click()` and says
## `changes captured`; `game` wrote the lambda, so there the word stands for its own `score` and it
## names nothing.

class Button:
    handlers: Array[fn() changes captured] = []

    fn on_click(owns f: fn() changes captured) changes handlers:
        this.handlers.push(f)

    fn click() changes handlers, captured:
        for ref mut h in this.handlers:
            h()

fn press(ref mut b: Button) changes captured:
    b.click()

fn game() -> int:
    score = 0
    b = Button()
    b.on_click(fn() changes score: score += 1)
    press(ref mut b)
    return score

fn main():
    println(game())
