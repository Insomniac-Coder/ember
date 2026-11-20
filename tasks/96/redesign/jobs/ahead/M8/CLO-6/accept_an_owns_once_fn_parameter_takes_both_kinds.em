# Clause CLO-6.1: "A parameter that takes a `once fn` holds it, `owns f: once fn(A) -> R`, and accepts
# both kinds"; the message's fix "`call_twice(fn(): consume(owns copy of s))`"
#$ test: run-pass
#$ rules: CLO-6, DIA-25
#$ stdout: a
#$ stdout: b
#$ stdout: c
#$ stdout: c
# A `once fn` and a plain function given to one `owns f: once fn()`; the message's line.

fn consume(owns s: String):
    println(s)

fn hello():
    println("b")

fn call_twice(f: fn()):
    f()
    f()

fn call_once(owns f: once fn()):
    f()

fn main():
    s = String("a")
    call_once(owns fn() owns s: consume(owns s))
    call_once(owns hello)
    c = String("c")
    call_twice(fn(): consume(owns copy of c))
