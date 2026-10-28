#$ test: run-pass
#$ rules: R12, R3
#$ stdout: 1
#$ stdout: 2
#$ stdout: 0
## Row 6, R12 (rules.md lines 512-513, 635-636; section 2, line 783, "nothing beyond the function value's
## own block"): `with copy of score` makes the handler's own copy, kept in its state across calls; the
## handler changes only its own state, a plain `fn()`, and `game`'s `score` stays 0. AN-53's
## `game_copy`. Checked by stdout, and by AddressSanitizer on the emitted C.

class Button:
    handlers: Array[fn() changes captured] = []

    fn on_click(owns f: fn() changes captured) changes handlers:
        this.handlers.push(f)

    fn click() changes handlers, captured:
        for ref mut h in this.handlers:
            h()

fn game_copy() -> int:
    score = 0
    b = Button()
    counted = fn() with copy of score:
        score += 1
        println(score)
    b.on_click(counted)
    b.click()
    b.click()
    return score

fn main():
    println(game_copy())
