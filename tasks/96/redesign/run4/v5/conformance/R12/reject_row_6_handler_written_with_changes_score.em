#$ test: compile-fail
#$ rules: GRM-2
#$ help: b.on_click(fn(): score += 1)
## design rules: R12, R5
## Verdict 86, row 6, R12 (rules.md lines 670-671): "A lambda writes nothing about what it changes:
## `b.on_click(fn(): score += 1)` changes `game`'s real `score` (row 6)". `changes` is not a word of Ember
## any more; a clause after a function's brackets is E0100 (rules.md section 3, line 1051), with the line
## to write. The nearest allowed program is accept_row_6_handler_changes_the_real_score.

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
    b.on_click(fn() changes score: score += 1)    #$ error[E0100]: `changes score` is not Ember: the compiler works out what this function changes from its body; write `b.on_click(fn(): score += 1)`
    b.click()
    return score

fn main():
    println(game())
