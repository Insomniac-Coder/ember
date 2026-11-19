#$ test: run-pass
#$ rules: DRP-5, OWN-6
#$ stdout: log
#$ stdout: 1
# `[DRP-5]` — a `drop` body takes a field out with `mem.take(ref mut this.f)`, on an `Option` or a
# `Default` field, leaving a value in its place.

class Writer:
    name: String = String("log")
    pending: Option[int] = Some(1)

    fn drop():
        name = mem.take(ref mut this.name)
        println(name)
        if mem.take(ref mut this.pending) exists as n:
            println(n)

fn main():
    _w = Writer()
