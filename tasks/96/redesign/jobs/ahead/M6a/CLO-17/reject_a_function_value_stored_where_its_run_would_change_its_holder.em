# Clause CLO-17.2: "a function value is not stored in a place that its body may reshape or end, or
# that is owned, all the way down, by anything its body may reshape or end, through any name it
# reaches: a real variable, (...) a parameter the call gives it (`ref mut u` in `fn(ref mut u:
# Ui)`), `this`, or a `static mut` (`BUS.handlers.push(owns fn(): BUS.handlers.clear())`), read
# through the chain of its calls (`[EFF-24]`)"
#$ test: compile-fail
#$ rules: CLO-17, CLO-18, EFF-24
# The first message is the spec's, with this file's line numbers. The others have no text in the
# spec: code only. `wipe_panels` ends the panel that holds the function (owned all the way down),
# through a call. The `weakref bus` capture case needs capture words (M8): not here. Q4 asks
# whether a function with no state (all but the first here) is refused too; these follow the
# clause's letter and its `BUS` example.

class Button:
    handlers: Array[fn()] = []

    fn on_click(owns f: fn()):
        this.handlers.push(owns f)

    fn click():
        for h in this.handlers:
            h()

class Bus:
    handlers: Array[fn()] = []

class Ui:
    handlers: Array[fn(ref mut Ui)] = []

class Panel:
    handlers: Array[fn()] = []

static mut BUS: Bus = Bus()
static mut PANELS: Array[Panel] = []

fn wipe_panels():
    PANELS.clear()

fn main():
    b = Button()
    b.on_click(owns fn(): b.handlers.clear())    #$ error[E3022]: this function clears `b.handlers`, and `on_click` would keep it in `b.handlers`, so it would change the button that holds it while the button runs it; clear the handlers after the click instead: `b.click()`, then `b.handlers.clear()`
    BUS.handlers.push(owns fn(): BUS.handlers.clear())    #$ error[E3022]
    ui = Ui()
    ui.handlers.push(owns fn(ref mut u: Ui): u.handlers.clear())    #$ error[E3022]
    PANELS.push(owns Panel())
    PANELS[0].handlers.push(owns wipe_panels)    #$ error[E3022]
