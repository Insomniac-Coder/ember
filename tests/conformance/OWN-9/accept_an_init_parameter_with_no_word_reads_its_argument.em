#$ test: run-pass
#$ rules: OWN-9
#$ stdout: a a
# `[OWN-9]` — an `init` parameter with no word reads its argument for the call only.

class Emitter:
    label: String

    fn init(name: String):
        this.label = copy of name

fn main():
    label = String("a")
    e = Emitter(label)
    println(label, e.label)
