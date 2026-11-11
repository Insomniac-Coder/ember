#$ test: compile-fail
#$ rules: OWN-9, FN-2a
# `[FN-2a]` — `owns` is not written before a constructor's argument the constructor already hands
# over: `Emitter(owns label)` is `E0109`, for a struct's `init` and a class's alike.

struct Label:
    text: String

    fn init(owns name: String):
        this.text = name

class Emitter:
    label: String

    fn init(owns name: String):
        this.label = name

fn main():
    a = String("x")
    l = Label(owns a)    #$ error[E0109]: `Label(...)` already hands `a` over; remove `owns`
    b = String("y")
    e = Emitter(owns b)    #$ error[E0109]: `Emitter(...)` already hands `b` over; remove `owns`
    println(l.text)
    println(e.label)
