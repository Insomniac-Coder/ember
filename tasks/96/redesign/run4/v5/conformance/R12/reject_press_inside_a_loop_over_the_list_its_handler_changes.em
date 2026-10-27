#$ test: compile-fail
#$ rules: BRW-18, CTL-2
#$ help: move line 29, `press(ref mut b)`, below the loop
## design rules: R12, R5, R8
## Verdict 86, R5 (rules.md lines 362-367, 379-383: "the message names the chain of calls"), R12 (lines
## 752-756): `press` runs the handlers through `b.click()`, and the handler `game` stored pushes to
## `names`; the compiler works out that chain with no word on any first line, so `press(ref mut b)` inside
## a loop over `names` is refused (E3020), the message naming each call on the way (R18). The nearest
## allowed program is accept_press_after_the_loop_over_the_list_its_handler_changes.

class Button:
    handlers: Array[fn()] = []

    fn on_click(owns f: fn()):
        this.handlers.push(f)

    fn click():
        for ref mut h in this.handlers:
            h()

fn press(ref mut b: Button):
    b.click()

fn game():
    names: Array[String] = [String("a")]
    b = Button()
    b.on_click(fn(): names.push(String("b")))
    for n in names:
        println(n)
        press(ref mut b)    #$ error[E3020]: `press(ref mut b)` (line 29) adds to `names` (through `press`, which calls `b.click()` on line 21, and `click`, line 16, which runs the handler stored on line 26, which pushes to `names`) while this loop walks `names` (line 27); press after the loop: move line 29, `press(ref mut b)`, below the loop

fn main():
    game()
