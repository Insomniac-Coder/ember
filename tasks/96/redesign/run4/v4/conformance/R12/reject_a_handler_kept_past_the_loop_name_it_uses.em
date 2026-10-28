#$ test: compile-fail
#$ rules: R12, R2
#$ help: b.on_click(fn() with copy of name: println(name))
## R12 (rules.md lines 776-777): a function value may not outlive a place it uses and does not own
## (E3026). AN-57 part 6: a lambda kept by a callee while it captures a view (here a `for` name, which
## ends with its round) gets "the same message shape, at the lambda's line, naming the captured view and
## the holder that keeps the lambda".

class Button:
    handlers: Array[fn()] = []

    fn on_click(owns f: fn()) changes handlers:
        this.handlers.push(f)

    fn click() changes handlers:
        for ref mut h in this.handlers:
            h()

fn main():
    names = [String("ann"), String("bo")]
    b = Button()
    for name in names:
        b.on_click(fn(): println(name))    #$ error[E3026]: the click handler uses `name`, a view of an element of `names` that ends with this round of the loop, and `b` keeps the handler after it; give the handler its own copy: `b.on_click(fn() with copy of name: println(name))`
    b.click()
