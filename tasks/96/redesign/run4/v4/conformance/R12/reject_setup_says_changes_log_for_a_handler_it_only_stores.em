#$ test: compile-fail
#$ rules: R12, R5
#$ help: fn setup(ref mut b: Button):
## R12 (rules.md lines 699-704): "Storing a function value is not a change"; `setup` hands the handler to
## `on_click`, which keeps it for `click` to run, so `setup` names nothing (G-06), and a first line is
## checked both ways (E2105). Message in the shape of AN-76 part 6's "`tick` says `changes LOG`".

static mut LOG: Array[String] = []

class Button:
    handlers: Array[fn() changes LOG] = []

    fn on_click(owns f: fn() changes LOG) changes handlers:
        this.handlers.push(f)

    fn click() changes handlers, LOG:
        for ref mut h in this.handlers:
            h()

fn setup(ref mut b: Button) changes LOG:    #$ error[E2105]: `setup` says `changes LOG` and never changes it: it only gives the handler to `b.on_click`, which keeps it, and storing a function is not a change of what it changes; remove `LOG` from its first line: `fn setup(ref mut b: Button):`
    b.on_click(fn() changes LOG: LOG.push(String("clicked")))

fn main() changes LOG:
    b = Button()
    setup(ref mut b)
    b.click()
    println(LOG.len())
