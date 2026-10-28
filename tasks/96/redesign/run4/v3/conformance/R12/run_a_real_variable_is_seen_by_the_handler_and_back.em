#$ test: run-pass
#$ rules: R12
#$ stdout: 1
#$ stdout: 11
## Row 6, R12 (rules.md lines 624-634; rules.md section 2, line 783, "nothing" for a captured variable
## at its address): the handler changes `game`'s real `score`, and a write by `game` is seen by the
## handler ("a write in the lambda is seen by the function, and back", area-D.md line 43). `b` never
## leaves `game`, so `score` stays at its address; where it lives never changes what the program does.
## Checked by stdout, and by a build of the emitted C under AddressSanitizer (AN-53 part 5's case 1).

class Button:
    handlers: Array[fn() changes captured] = []

    fn on_click(owns f: fn() changes captured) changes handlers:
        this.handlers.push(f)

    fn click() changes handlers, captured:
        for ref mut h in this.handlers:
            h()

fn game():
    score = 0
    b = Button()
    b.on_click(fn() changes score: score += 1)
    b.click()
    println(score)
    score = 10
    b.click()
    println(score)

fn main():
    game()
