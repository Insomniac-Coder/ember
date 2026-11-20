# Clause CLO-18.1: "A call through a stored function value whose functions may change their own
# state is a change of the place that holds it: the holder is reached through a name that may
# change it (`for ref mut h in this.handlers: h()`), and a plain `h` there is `E3023`"
#$ test: compile-fail
#$ rules: CLO-18, BRW-15, CLO-10
# The spec's message, with this file's line numbers. The handler is put in the list directly (Q2).
# Its own `count` needs capture words (M8).

class Button:
    handlers: Array[fn()] = []

    fn click():
        for h in this.handlers:
            h()    #$ error[E3023]: `h` only reads the handler in `this.handlers`, and calling it may change its own state (the handler stored on line 19 adds to its own `count`); walk the handlers with `for ref mut h in this.handlers:` in place of line 13

fn main():
    count = 0
    b = Button()
    b.handlers.push(owns fn() copy of count: count += 1)
    b.click()
