#$ test: run-pass
#$ rules: GRM-3
#$ stdout: 4
## `[GRM-3]` — library and user generic types are ordinary paths, and generic
## arguments may sit on any segment: `Wrap[int].make`.

struct Wrap[T]:
    value: T

    fn make(v: T) -> Wrap[T]:
        return Wrap[T](v)

fn main():
    w = Wrap[int].make(4)
    println(w.value)
