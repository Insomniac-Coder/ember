#$ test: compile-fail
#$ rules: WK-16, OBJ-1
# `[WK-16]` — a `weakref` of a value held `owns dyn I` is `E2119`: its kind is known only while the
# program runs, and whether its header holds a weak count is its class's to fix.
interface Listener:
    fn notify(msg: StrView)

class Button implements Listener:
    fn notify(msg: StrView):
        println(msg)

fn main():
    held: owns dyn Listener = owns Button()
    w = weakref held                  #$ error[E2119]: `held` holds an `owns dyn Listener`, a kind known only while the program runs, and a `weakref` link needs a class that fixes its header; link to a base class: write `open class Listener:` with `virtual fn notify(msg: StrView): pass` in place of lines 5 and 6, and `class Button(Listener):` in place of line 8
