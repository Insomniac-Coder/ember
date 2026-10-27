#$ test: compile-fail
#$ rules: CLO-6
#$ help: owns f: once fn()
## design rules: R12, R4
## R12 (rules.md lines 782-784): "A parameter that takes one holds it, `owns f: once fn()`. A plain
## `f: once fn()` ... [is] refused (E3013)". rules.md section 3's line 1096 third message, word for word.
## The nearest allowed program is accept_call_once_holds_an_owns_once_fn_and_call_twice_hands_over_copies.

fn call_once(f: once fn()):    #$ error[E3013]: `call_once` uses `f` up, so it must hold it: write `owns f: once fn()`
    f()

fn consume(owns s: String):
    println(s)

fn main():
    s = String("a")
    call_once(fn() owns s: consume(s))
