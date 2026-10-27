#$ test: run-pass
#$ rules: CLO-1, FN-4, BRW-16
#$ stdout: 1
## design rules: R12, R5, R8
## Row 6, verdict 86, R12 (rules.md lines 670-671, 752-756): "`b.on_click(fn(): score += 1)` changes
## `game`'s real `score` (row 6)"; no word on the lambda, on `click` or on `game`. The click adds 1 to the
## real `score`. AN-53's NEW program.

class Button:
    handlers: Array[fn()] = []

    fn on_click(owns f: fn()):
        this.handlers.push(f)

    fn click():
        for ref mut h in this.handlers:
            h()

fn game() -> int:
    score = 0
    b = Button()
    b.on_click(fn(): score += 1)
    b.click()
    return score

fn main():
    println(game())
