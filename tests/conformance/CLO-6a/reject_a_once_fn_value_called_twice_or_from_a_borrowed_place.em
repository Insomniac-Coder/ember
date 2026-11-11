#$ test: compile-fail
#$ rules: CLO-6a, CLO-6, OWN-3
## `[CLO-6a]` — calling a `once fn` value moves it out of its place, so the move rules decide where
## that may happen: not twice (`E3040`), not in a loop (`E3040`, its later round), not through a borrowed parameter
## (`E3013`), not out of a collection's element in place (`E3011`: the collection's API removes
## it, `pop_front`), not out of a class's field (`E3013`).

struct Task:
    run: once fn() -> int

class Holder:
    cb: once fn() -> int

    fn fire() -> int:
        return this.cb()    #$ error[E3013]: `self` is a `Holder` object, which other names may reach, so `self.cb` cannot be taken out and leave a hole; take it and leave a value in its place: `mem.take(ref mut self.cb)`

fn consume(owns s: String) -> int:
    return s.len()

fn through_borrow(t: Task) -> int:
    return t.run()    #$ error[E3013]: cannot move out of a borrowed parameter

fn main():
    s = "x".to_string()
    f: once fn() -> int = owned fn() => consume(owns s)
    println(f())
    println(f())    #$ error[E3040]: `f` is a `once fn`: its call on line 26 uses it up, and it is used again on line 27; a `once fn` runs once (take `f: fn(…)` where it is called more than once)
    g: once fn() -> int = fn() => 3
    for i in 0..2:
        println(g())    #$ error[E3040]: `g` is handed over on line 30 and the loop comes round again, so the next round finds it gone; give it a new value before the next round (`g = …`), declare it inside the loop, or hand over a copy of your own: `copy of g`
    jobs: Array[once fn() -> int] = []
    jobs.push(owns fn() => 1)
    println(jobs[0]())    #$ error[E3011]: cannot move out of an element
