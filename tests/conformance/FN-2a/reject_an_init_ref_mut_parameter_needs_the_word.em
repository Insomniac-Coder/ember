#$ test: compile-fail
#$ rules: FN-2a
#$ help: say so at the call: `w = Widget(ref mut reg, String("x"))`
# `[FN-2a]` — `ref mut` is written before an argument for an `init`'s `ref mut` parameter as at
# every call: without it `E3028`.

struct Registry:
    n: int

struct Widget:
    id: int

    fn init(ref mut registry: Registry, owns name: String):
        registry.n += 1
        this.id = registry.n

fn main():
    reg = Registry(n=0)
    w = Widget(reg, String("x"))    #$ error[E3028]: `Widget` may change `reg`
    println(w.id)
