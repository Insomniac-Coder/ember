#$ test: run-pass
#$ rules: CLO-1, OWN-8
#$ stdout: 1
#$ stdout: 2
#$ stdout: 0
## design rules: R12, R3
## Row 6, R12 (rules.md lines 681-683, 766-767; section 2, line 985, "nothing beyond the function value's
## own block"): `copy of score` makes the handler's own copy, kept in its state across calls, and
## `game`'s `score` stays 0. AN-53's `game_copy`. Checked by stdout, and by AddressSanitizer on the
## emitted C.

class Button:
    handlers: Array[fn()] = []

    fn on_click(owns f: fn()):
        this.handlers.push(f)

    fn click():
        for ref mut h in this.handlers:
            h()

fn game_copy() -> int:
    score = 0
    b = Button()
    counted = fn() copy of score:
        score += 1
        println(score)
    b.on_click(counted)
    b.click()
    b.click()
    return score

fn main():
    println(game_copy())
