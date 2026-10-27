#$ test: compile-fail
#$ rules: GRM-2
#$ help: b.on_click(fn() copy of name: println(name))
## design rules: R12, R3
## Verdict 83, R3 (rules.md lines 230-234) and R12 (lines 766-767): a lambda's capture words follow its
## brackets on their own (`fn() copy of a, owns b, ...:`); `with` keeps one meaning, a block. `with` before
## the captures is E0100 (rules.md section 3, line 1051: "a function's captures follow its brackets with
## no `with`"). The nearest allowed program is accept_the_handler_takes_a_copy_of_the_loop_name.

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
        b.on_click(fn() with copy of name: println(name))    #$ error[E0100]: a function's captures follow its brackets with no `with`: write `b.on_click(fn() copy of name: println(name))`
    b.click()
