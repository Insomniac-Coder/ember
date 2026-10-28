#$ test: compile-fail
#$ rules: R12, R8
#$ help: call `b.click()` after the `for` loop
## R12 (rules.md lines 626-628): in the function that wrote the lambda, a place that holds it counts as
## reaching every real variable it captures, and a call that may run it as a change of each one it
## changes, so `b.click()` is refused inside a loop over `names` (AN-53 part 6, "The same holds for
## `b.click()`"). Message in the shape of rules.md section 3's line 836.

class Button:
    handlers: Array[fn() changes captured] = []

    fn on_click(owns f: fn() changes captured) changes handlers:
        this.handlers.push(f)

    fn click() changes handlers, captured:
        for ref mut h in this.handlers:
            h()

fn main():
    names: Array[String] = [String("a")]
    b = Button()
    b.on_click(fn() changes names: names.push(String("b")))
    for n in names:
        println(n)
        b.click()    #$ error[E3020]: this loop walks `names` (line 23), and `b.click()` (line 25) runs the handler stored in `b` on line 22, which pushes to `names` (`fn() changes names`); click after the loop
