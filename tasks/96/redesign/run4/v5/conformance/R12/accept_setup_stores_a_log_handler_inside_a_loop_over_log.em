#$ test: run-pass
#$ rules: R12, R5, R8
#$ stdout: start
#$ stdout: 2
## Verdict 86, R12 (rules.md lines 735-739): "Storing a function value is not a change: putting it in a
## field, a list, a local or an iterator adapter ... changes nothing the function changes; the call that
## later runs it does". `setup` hands a handler that pushes to `LOG` to `on_click`, which keeps it, so
## `setup` inside the loop over `LOG` changes nothing the loop walks (G-06); `b.click()`, which runs it,
## comes after the loop. The click inside the loop is refused
## (reject_clicking_a_log_button_inside_a_loop_over_log).

static mut LOG: Array[String] = []

class Button:
    handlers: Array[fn()] = []

    fn on_click(owns f: fn()):
        this.handlers.push(f)

    fn click():
        for ref mut h in this.handlers:
            h()

fn setup(ref mut b: Button):
    b.on_click(fn(): LOG.push(String("clicked")))

fn main():
    LOG.push(String("start"))
    b = Button()
    for entry in LOG:
        println(entry)
        setup(ref mut b)
    b.click()
    println(LOG.len())
