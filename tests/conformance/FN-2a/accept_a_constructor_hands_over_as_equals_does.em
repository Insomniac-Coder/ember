#$ test: run-pass
#$ rules: FN-2a, OWN-9
#$ stdout: a
#$ stdout: a
#$ stdout: b
#$ stdout: c
#$ stdout: c
#$ stdout: 1
# `[FN-2a]` — a value given to a memberwise constructor, or to an `owns` parameter of a written
# `init`, is handed over as `=` does and takes no `owns` (`Emitter(copy of label)` keeps it); a
# value given to an `init` parameter with no word is read for the call; and `ref mut` is written
# before an argument for an `init`'s `ref mut` parameter as at every call.

struct Emitter:
    name: String

    fn init(owns name: String):
        this.name = name

struct Label:
    text: String

    fn init(text: String):
        this.text = copy of text

struct Registry:
    n: int

struct Widget:
    id: int

    fn init(ref mut registry: Registry, owns name: String):
        registry.n += 1
        this.id = registry.n

fn main():
    label = String("a")
    e = Emitter(copy of label)
    println(label)
    println(e.name)
    other = String("b")
    e2 = Emitter(other)
    println(e2.name)
    caption = String("c")
    l = Label(caption)
    println(caption)
    println(l.text)
    reg = Registry(n=0)
    w = Widget(ref mut reg, String("x"))
    println(w.id)
