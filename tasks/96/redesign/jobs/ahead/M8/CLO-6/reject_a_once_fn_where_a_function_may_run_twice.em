# Clause CLO-6.1: "A function that gives one of its captures away (to an `owns` parameter, a return, a
# field) can be called only once. Its type is `once fn(A) -> R` (...); given where a function may run
# twice it is `E3013`"; "a plain `f: once fn(A) -> R` is `E3013`"; "Calling `f` uses it up; a second
# call is `E3040`."
#$ test: compile-fail
#$ rules: CLO-6, OWN-3
# The spec's two `E3013` messages; the second call is code only.

fn consume(owns s: String):
    println(s)

fn call_twice(f: fn()):
    f()
    f()

fn call_once(f: once fn()):    #$ error[E3013]: `call_once` uses `f` up, so it must hold it: write `owns f: once fn()`
    f()

fn run_twice(owns f: once fn()):
    f()
    f()    #$ error[E3040]

fn main():
    s = String("a")
    call_twice(fn() owns s: consume(owns s))    #$ error[E3013]: this function gives `s` away, so it can run only once, and `call_twice` runs it twice; hand over a copy each time instead: `call_twice(fn(): consume(owns copy of s))`
    t = String("b")
    call_once(owns fn() owns t: consume(owns t))
    u = String("c")
    run_twice(owns fn() owns u: consume(owns u))
