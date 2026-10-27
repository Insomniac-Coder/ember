#$ test: compile-fail
#$ rules: R12, R4
#$ help: fn call_once(owns f: once fn()):
## R12 (rules.md lines 648-650): "A parameter that takes one holds it, `owns f: once fn()`. A plain
## `f: once fn()` ... [is] refused (E3013)". rules.md section 3's line 865 message, word for word.

fn call_once(f: once fn()):    #$ error[E3013]: `call_once` uses `f` up, so it must hold it: write `owns f: once fn()`
    f()

fn consume(owns s: String):
    println(s)

fn main():
    s = String("a")
    call_once(fn() with owns s: consume(s))
