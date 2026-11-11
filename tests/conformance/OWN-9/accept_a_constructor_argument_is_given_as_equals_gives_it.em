#$ test: run-pass
#$ rules: OWN-9, FN-2a
#$ stdout: x
#$ y
#$ 5
#$ 5
# `[OWN-9]`, `[FN-2a]` (verdict 159) — a constructor's argument takes no word: a value given to an
# `owns` parameter of a written `init` is handed over as `=` does, a struct's `init` and a class's
# alike, and a value `=` copies is copied, so `Counter(k)` keeps `k`.

struct Label:
    text: String

    fn init(owns name: String):
        this.text = name

class Emitter:
    label: String

    fn init(owns name: String):
        this.label = name

class Counter:
    n: int

    fn init(owns start: int):
        this.n = start

fn main():
    a = String("x")
    l = Label(a)
    println(l.text)
    b = String("y")
    e = Emitter(b)
    println(e.label)
    k = 5
    c = Counter(k)
    println(k)
    println(c.n)
