# Clause EFF-2.1: "A contract is written on a function type above a `type` line that names one
# (verdict 106): `@noalloc` above `type Handler = fn()`; every function stored into a place of that
# type is checked against it, with the contract's code"
#$ test: compile-fail
#$ rules: EFF-2, EFF-5, ATT-1
# `log_it` allocates, so storing it in a `Handler` field, list or local is `E4001` at each store.
# No text in the spec for these: code only.

@noalloc
type Handler = fn()

static mut LOG: Array[String] = []

fn log_it():
    LOG.push(owns String("it"))

fn quiet():
    pass

class Button:
    on_click: Handler = quiet
    extra: Array[Handler] = []

fn main():
    b = Button()
    b.on_click = log_it    #$ error[E4001]
    b.extra.push(owns log_it)    #$ error[E4001]
    h: Handler = log_it    #$ error[E4001]
    h()
