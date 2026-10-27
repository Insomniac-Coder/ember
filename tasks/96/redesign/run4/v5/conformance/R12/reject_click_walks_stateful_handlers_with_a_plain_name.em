#$ test: compile-fail
#$ rules: R12, R5
#$ help: walk the handlers with `for ref mut h in this.handlers:`
## R12 (rules.md lines 776-779): "A call through a stored function value whose functions may change their
## own state is a change of the place that holds it: the holder is reached through a name that may change
## it (`for ref mut h in this.handlers: h()`; E3023 for a plain `h`)". The handler the program stores keeps
## its own copy of `n` and changes it, worked out from its body (verdict 86); one mistake, one message.
## rules.md section 3's line 1042 first message, in this program's lines. The nearest allowed program is
## accept_click_walks_stateful_handlers_with_ref_mut.

class Button:
    handlers: Array[fn()] = []

    fn on_click(owns f: fn()):
        this.handlers.push(f)

    fn click():
        for h in this.handlers:
            h()    #$ error[E3023]: `h` only reads the handler in `this.handlers`, and calling it may change its own state (`tick`, stored on line 27, adds to its own copy of `n`, line 25); walk the handlers with `for ref mut h in this.handlers:` in place of line 18

fn main():
    b = Button()
    n = 0
    tick = fn() copy of n:
        n += 1
        println(n)
    b.on_click(tick)
    b.click()
