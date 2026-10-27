#$ test: compile-fail
#$ rules: R12, R8
#$ help: move line 27, `b.click()`, below the loop
## R12 (rules.md lines 752-756): in the function that wrote the lambda, "a place that holds the function
## value counts as reaching every real variable it captures, and a call that may run it as a change of
## each one it changes", so `b.click()` is refused inside a loop over `names` (AN-53 part 6, "The same
## holds for `b.click()`"). rules.md section 3's line 1065 `b.click()` message shape, in this program's
## lines (F-12). The nearest allowed program is
## accept_clicking_after_the_loop_over_the_list_a_handler_changes.

class Button:
    handlers: Array[fn()] = []

    fn on_click(owns f: fn()):
        this.handlers.push(f)

    fn click():
        for ref mut h in this.handlers:
            h()

fn main():
    names: Array[String] = [String("a")]
    b = Button()
    b.on_click(fn(): names.push(String("b")))
    for n in names:
        println(n)
        b.click()    #$ error[E3020]: `b.click()` (line 27) adds to `names` (through `click`, line 17, which runs the handler stored on line 24, which pushes to `names`) while this loop walks `names` (line 25); click after the loop: move line 27, `b.click()`, below the loop
