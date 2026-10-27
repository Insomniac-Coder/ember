#$ test: compile-fail
#$ rules: R12, R5
#$ help: fn click() changes handlers:
## R12 (rules.md lines 641-644): a call through a stored function value is a change of the place that
## holds it, so the holder is reached through a name that may change it (E2105 for the first line,
## E3023 for the name). rules.md section 3's line 866 message, word for word: one mistake, one message.

class Button:
    handlers: Array[fn()] = []

    fn on_click(owns f: fn()) changes handlers:
        this.handlers.push(f)

    fn click():    #$ error[E2105]: `click` calls the handlers stored in `this.handlers`, which may change their own state; say so on its first line, `fn click() changes handlers:`, and walk them with `for ref mut h in this.handlers:`
        for h in this.handlers:
            h()

fn main():
    b = Button()
    n = 0
    tick = fn() with copy of n:
        n += 1
        println(n)
    b.on_click(tick)
    b.click()
