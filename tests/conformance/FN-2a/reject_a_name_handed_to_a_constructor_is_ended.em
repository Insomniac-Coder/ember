#$ test: compile-fail
#$ rules: FN-2a, OWN-9
# `[FN-2a]` — a value given to a constructor is handed over as `=` does: the caller's name ends,
# and a later use is `E3040`; `Emitter(copy of label)` keeps it.

struct Emitter:
    name: String

    fn init(owns name: String):
        this.name = name

fn main():
    label = String("a")
    e = Emitter(label)
    println(e.name)
    println(label)    #$ error[E3040]
