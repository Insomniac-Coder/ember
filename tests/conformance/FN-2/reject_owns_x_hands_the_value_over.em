#$ test: compile-fail
#$ rules: FN-2, FN-2a
# `[FN-2]` — a callee that needs a value of its own takes it `owns`, and the caller passes `owns x`
# to hand it over: its name ends there.

fn keep(owns s: String):
    println(s)

fn main():
    s = String("ab")
    keep(owns s)
    println(s)    #$ error[E3040]: `s` was handed over to `keep`
