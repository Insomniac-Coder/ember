# Clause CLO-6a.1: "A `once fn` value is called out of a place that owns it and that no other name
# reaches: a local, a part of a struct local (...), or a collection element the calling API removes
# (`jobs.pop_front()()`); the call hands the value out of its place"; the message's fix
#$ test: run-pass
#$ rules: CLO-6a, DIA-25
#$ stdout: a
#$ stdout: b
#$ stdout: c
# A local, a list element taken out by `pop`, and the message's two lines for a class field.

fn consume(owns s: String):
    println(s)

class Job:
    task: once fn()

    fn init(owns task: once fn()):
        this.task = task

fn main():
    a = String("a")
    f: once fn() = fn() owns a: consume(owns a)
    f()
    b = String("b")
    jobs: Array[once fn()] = []
    jobs.push(owns fn() owns b: consume(owns b))
    jobs.pop().unwrap()()
    c = String("c")
    j = Job(fn() owns c: consume(owns c))
    t = mem.replace(ref mut j.task, fn(): pass)
    t()
