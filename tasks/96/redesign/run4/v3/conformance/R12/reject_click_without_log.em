#$ test: compile-fail
#$ rules: R12, R5
#$ help: fn click() changes handlers, LOG:
## Verdict 17, R12 (rules.md lines 553-556): a call through a function value makes the changes its type
## says, and the function that makes the call names them on its first line. AN-76 part 6; rules.md
## section 3's line 819 message, word for word.

static mut LOG: Array[String] = []

type Handler = fn() changes LOG

class Button:
    handlers: Array[Handler] = []

    fn on_click(owns f: Handler) changes handlers:
        this.handlers.push(f)

    fn click() changes handlers:    #$ error[E2105]: `click` runs the handlers in `this.handlers`, whose type says `changes LOG`; add it to its first line: `fn click() changes handlers, LOG:`
        for ref mut h in this.handlers:
            h()

fn main() changes LOG:
    b = Button()
    b.on_click(fn() changes LOG: LOG.push(String("clicked")))
    b.click()
    println(LOG.len())
