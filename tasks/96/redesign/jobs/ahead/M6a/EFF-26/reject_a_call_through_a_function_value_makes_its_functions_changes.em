# Clause EFF-26.1: "A call through a function value makes the changes of every function it may run"
#$ test: compile-fail
#$ rules: EFF-26, CLO-17, BRW-18, EFF-2
# A local function value, a field and a parameter of a callable type: each may run `log_it`, which
# pushes to `LOG`, inside a walk of `LOG`. No text in the spec for these: code only.

static mut LOG: Array[String] = []

fn log_it():
    LOG.push(owns String("it"))

fn quiet():
    pass

struct Hook:
    run: fn() = quiet

fn call_it(f: fn()):
    f()

fn main():
    LOG.push(owns String("start"))
    h: fn() = quiet
    h = log_it
    k = Hook()
    k.run = log_it
    for line in LOG:
        println(line)
        h()    #$ error[E3020]
        k.run()    #$ error[E3020]
        call_it(log_it)    #$ error[E3020]
