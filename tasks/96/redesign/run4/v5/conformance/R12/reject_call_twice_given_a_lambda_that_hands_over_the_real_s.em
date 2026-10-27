#$ test: compile-fail
#$ rules: OWN-11, CLO-6
#$ help: call_twice(fn(): consume(copy of s))
## design rules: R12, R2
## R12 (rules.md lines 752-753, 782-784), row 6: a capture with no word is the real variable, which `main`
## keeps owning; a function value cannot hand it over (E3013, a hand-over out of a name that does not own
## the value; rules.md section 3, line 1096). AN-59 part 6's message, with the one way out that compiles
## here (its other, `owns s`, is refused itself: `call_twice` runs the function twice; verdict 73).

fn call_twice(f: fn()):
    f()
    f()

fn consume(owns s: String):
    println(s)

fn main():
    s = String("a")
    call_twice(fn(): consume(s))    #$ error[E3013]: `consume(s)` would hand over `main`'s `s` from inside a function `call_twice` may run twice, and the function does not own `s`; hand over a copy each time: `call_twice(fn(): consume(copy of s))`
