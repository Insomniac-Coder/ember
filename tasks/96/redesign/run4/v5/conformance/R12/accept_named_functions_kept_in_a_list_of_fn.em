#$ test: run-pass
#$ rules: R12, R5
#$ stdout: 1
## Verdict 86, R12 (rules.md lines 662-665, 674-675): "A named function is a value of its type
## (`fn log_click():` is a `fn()`)"; a function that pushes to `LOG` and one that changes nothing are kept
## in one `Array[fn()]`, and `main`, which runs them, writes nothing: what the call changes is worked out
## from the two functions stored there.

static mut LOG: Array[String] = []

fn log_click():
    LOG.push(String("clicked"))

fn quiet():
    pass

class Button:
    handlers: Array[fn()] = []

fn main():
    b = Button()
    b.handlers.push(log_click)
    b.handlers.push(quiet)
    for ref mut h in b.handlers:
        h()
    println(LOG.len())
