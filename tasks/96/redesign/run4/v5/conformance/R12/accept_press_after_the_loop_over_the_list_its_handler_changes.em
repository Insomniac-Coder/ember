#$ test: run-pass
#$ rules: R12, R5, R8
#$ stdout: a
#$ stdout: 2
## Verdict 86, R5, R12 (rules.md lines 752-756): `press` runs the handlers through `b.click()`; neither
## `press`, `click` nor `game` writes what they change. The walk of `names` ends first, then `press`
## runs the handler that pushes to it. The nearest allowed program to
## reject_press_inside_a_loop_over_the_list_its_handler_changes.

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
    press(ref mut b)
    println(names.len())

fn main():
    game()
