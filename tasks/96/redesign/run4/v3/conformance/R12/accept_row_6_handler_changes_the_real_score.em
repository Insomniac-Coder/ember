#$ test: run-pass
#$ rules: R12, R5, R8
#$ stdout: 1
## Row 6, verdicts 22 and 35, R12 (rules.md lines 505-507, 524-535, 563-569, 624-628): the handler names
## `game`'s real `score`, the button's list covers it with `captured`, `click` repeats that word, and
## `game`, which wrote the lambda, names nothing (its own local). The click adds 1 to the real `score`.
## AN-53's NEW program.

class Button:
    handlers: Array[fn() changes captured] = []

    fn on_click(owns f: fn() changes captured) changes handlers:
        this.handlers.push(f)

    fn click() changes handlers, captured:
        for ref mut h in this.handlers:
            h()

fn game() -> int:
    score = 0
    b = Button()
    b.on_click(fn() changes score: score += 1)
    b.click()
    return score

fn main():
    println(game())
