#$ test: compile-fail
#$ rules: R12, R2
#$ help: b.on_click(fn() copy of name: println(name))
## R12 (rules.md lines 752-759, 785-786): "A function value may not outlive a place it uses and does not
## own (E3026)". The handler uses the real loop name `name`, a view of this round's element, and `b` keeps
## the handler after the round ends. AN-57 part 6's message shape, at the lambda's line (rules.md section
## 3, line 1095), with the capture word after the brackets (verdict 83). The nearest allowed program is
## accept_the_handler_takes_a_copy_of_the_loop_name.

class Button:
    handlers: Array[fn()] = []

    fn on_click(owns f: fn()):
        this.handlers.push(f)

    fn click():
        for ref mut h in this.handlers:
            h()

fn main():
    names = [String("ann"), String("bo")]
    b = Button()
    for name in names:
        b.on_click(fn(): println(name))    #$ error[E3026]: the click handler uses `name`, a view of an element of `names` that ends with this round of the loop, and `b` keeps the handler after it; give the handler its own copy: `b.on_click(fn() copy of name: println(name))`
    b.click()
