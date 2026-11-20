# Clause EFF-26.4: "A generic body's calls of its type parameters' methods make the changes those
# methods make for the types it is given, worked out for each instance."
#$ test: compile-fail
#$ rules: EFF-26, EFF-24, MONO-1
# `run(Quiet())` inside the walk is allowed: its instance changes nothing. `run(Loud())` pushes to
# `LOG`. No text in the spec: code only.

static mut LOG: Array[String] = []

interface Tick:
    fn tick()

struct Loud implements Tick:
    fn tick():
        LOG.push(owns String("loud"))

struct Quiet implements Tick:
    fn tick():
        pass

[T: needs Tick]
fn run(t: T):
    t.tick()

fn main():
    LOG.push(owns String("start"))
    for line in LOG:
        println(line)
        run(Quiet())
        run(Loud())    #$ error[E3020]
