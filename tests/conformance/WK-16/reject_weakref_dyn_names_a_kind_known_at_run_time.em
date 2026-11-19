#$ test: compile-fail
#$ rules: WK-16, OBJ-1, TYP-45
# `[WK-16]` — the type `weakref dyn I` is `E2119`: a link to a base class is one pointer, opened with
# one header test, and every object of the family carries the count (the rule's own example).
interface Listener:
    fn notify(msg: StrView)

class Hub:
    listeners: Array[weakref dyn Listener] = []    #$ error[E2119]: `weakref dyn Listener` (line 9) names a kind known only while the program runs, and a `weakref` link needs a class that fixes its header; link to a base class: write `open class Listener:` with `virtual fn notify(msg: StrView): pass` in place of lines 5 and 6, `class Button(Listener):` in place of line 11, and `listeners: Array[weakref Listener] = []` in place of line 9

class Button implements Listener:
    fn notify(msg: StrView):
        println(msg)

fn main():
    h = Hub()
    println(h.listeners.len())
