#$ test: compile-fail
#$ rules: CLO-6
## `[CLO-6]` - a function that gives `s` away runs once; given where it may run twice it is `E3013`,
## and the help is the line to write: the call with a copy given away each time.

fn consume(owns s: String):
    pass

[F: needs fn()]
fn call_twice(f: F):
    f()
    f()

fn main():
    s = String.from("x")
    call_twice(owned fn() => consume(owns s))    #$ error[E3013]: this function gives `s` away, so it can run only once, and `call_twice` may run it more than once
#$ help: hand over a copy each time instead: `call_twice(owned fn() => consume(owns copy of s))`
