#$ test: run-pass
#$ rules: R12, R3
#$ stdout: ann
#$ stdout: bo
## R12 (rules.md lines 681-683, 766-767): "A named capture (`copy of`, ...) is the function value's own
## state", so the handler keeps its own copy of each round's name and outlives nothing it uses (E3026 is
## for the name with no word: reject_a_handler_kept_past_the_loop_name_it_uses). Verdict 83: the capture
## word follows the brackets, with no `with` (reject_with_before_a_lambdas_captures).

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
        b.on_click(fn() copy of name: println(name))
    b.click()
