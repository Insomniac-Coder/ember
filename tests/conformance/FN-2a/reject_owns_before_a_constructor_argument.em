#$ test: compile-fail
#$ rules: FN-2a, OWN-9
# `[FN-2a]` — `Emitter(owns label)` is `E0109`: the constructor already hands `label` over.

struct Emitter:
    name: String

    fn init(owns name: String):
        this.name = name

fn main():
    label = String("a")
    e = Emitter(owns label)    #$ error[E0109]: `Emitter(...)` already hands `label` over; remove `owns`
    println(e.name)
