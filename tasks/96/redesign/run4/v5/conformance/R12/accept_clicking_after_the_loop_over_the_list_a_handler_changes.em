#$ test: run-pass
#$ rules: BRW-18, CTL-2
#$ stdout: a
#$ stdout: 2
## design rules: R12, R8
## R12 (rules.md lines 752-756), R8 (c): the click runs after the walk of `names`; the handler pushes to
## `main`'s real `names`, and no line says so. The nearest allowed program to
## reject_clicking_inside_a_loop_over_a_list_a_handler_changes.

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
    b.click()
    println(names.len())
