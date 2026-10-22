#$ test: compile-fail
#$ rules: CLO-6a, CLO-6, OWN-3
## `[CLO-6a]` — calling a `once fn` value moves it out of its place, so the move rules decide where
## that may happen: not twice (`E3040`), not in a loop (`E3041`), not through a borrowed parameter
## (`E3013`), not out of a collection's element in place (`E3011`: the collection's API removes
## it, `pop_front`), not out of a class's field (`E3012`).

struct Task:
    run: once fn() -> int

class Holder:
    cb: once fn() -> int

    fn fire(self) -> int:
        return self.cb()    #$ error[E3012]: cannot move out of a class field

fn consume(owned s: String) -> int:
    return s.len()

fn through_borrow(t: Task) -> int:
    return t.run()    #$ error[E3013]: cannot move out of a borrowed parameter

fn main():
    s = "x".to_string()
    f: once fn() -> int = owned fn() => consume(s)
    println(f())
    println(f())    #$ error[E3040]: `f` has been moved out of
    g: once fn() -> int = fn() => 3
    for i in 0..2:
        println(g())    #$ error[E3041]: `g` is moved in a loop
    jobs: Array[once fn() -> int] = []
    jobs.push(fn() => 1)
    println(jobs[0]())    #$ error[E3011]: cannot move out of an element
