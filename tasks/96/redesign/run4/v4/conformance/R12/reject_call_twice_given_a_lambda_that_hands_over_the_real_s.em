#$ test: compile-fail
#$ rules: R12, R2
#$ help: call_twice(fn(): consume(copy of s))
## R12 (rules.md lines 741-745, 773-775), row 6: a capture with no word is the real variable, which
## `main` keeps owning; a function value cannot hand it over (E3013, a hand-over out of a name that does
## not own the value). AN-59 part 6's message, corrected: its first way out (`with owns s`) was refused
## itself here (E3013, `call_twice` runs it twice), so the message gives the one that compiles (verdict 73).

fn call_twice(f: fn()):
    f()
    f()

fn consume(owns s: String):
    println(s)

fn main():
    s = String("a")
    call_twice(fn(): consume(s))    #$ error[E3013]: `consume(s)` would hand over `main`'s `s` from inside a function `call_twice` may run twice, and the function does not own `s`; hand over a copy each time: `call_twice(fn(): consume(copy of s))`
