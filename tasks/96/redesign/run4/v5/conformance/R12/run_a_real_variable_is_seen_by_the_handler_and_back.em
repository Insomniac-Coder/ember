#$ test: run-pass
#$ rules: CLO-1
#$ stdout: 1
#$ stdout: 11
## design rules: R12
## Row 6, R12 (rules.md lines 752-759; section 2, line 985, "nothing" for a captured real variable at
## its own address): the handler changes `game`'s real `score`, and a write by `game` is seen by the
## handler ("a write in the lambda is seen by the function, and back", area-D.md line 43). "Without a
## word a function only borrows: a real variable stays at its own address, and the function value must
## provably end before it": `b`, which keeps the handler, never leaves `game`.
## Checked by stdout, and by a build of the emitted C under AddressSanitizer (AN-53 part 5's case 1).

class Button:
    handlers: Array[fn()] = []

    fn on_click(owns f: fn()):
        this.handlers.push(f)

    fn click():
        for ref mut h in this.handlers:
            h()

fn game():
    score = 0
    b = Button()
    b.on_click(fn(): score += 1)
    b.click()
    println(score)
    score = 10
    b.click()
    println(score)

fn main():
    game()
