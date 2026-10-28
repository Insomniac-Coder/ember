#$ test: run-pass
#$ rules: R12, R8
#$ stdout: a
#$ stdout: 2
## R12 (rules.md lines 743-745), R8 (c): the click runs after the walk of `names`; the handler pushes to
## `main`'s real `names`, which `main` wrote and so names nothing for.

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
    b.click()
    println(names.len())
