# Clause EFF-2.1: "and a call through such a place has no effect the contract excludes"
#$ test: run-pass
#$ rules: EFF-2, EFF-5
#$ stdout: 2
# `run` is `@noalloc` and calls through a `Handler`, whatever the program stores there.

@noalloc
type Handler = fn()

static mut COUNT: int = 0

fn bump():
    COUNT += 1

class Button:
    on_click: Handler = bump

@noalloc
fn run(b: Button):
    b.on_click()

fn main():
    b = Button()
    run(b)
    run(b)
    println(COUNT)
