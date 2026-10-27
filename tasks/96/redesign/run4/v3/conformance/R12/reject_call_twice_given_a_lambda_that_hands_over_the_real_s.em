#$ test: compile-fail
#$ rules: R12, R2
#$ help: call_twice(fn(): consume(copy of s))
## R12 (rules.md lines 624-628, 648-650), row 6: a capture with no word is the real variable, which
## `main` keeps owning; a function value cannot hand it over (E3013, a hand-over out of a name that does
## not own the value). AN-59 part 6's message, word for word.

fn call_twice(f: fn()):
    f()
    f()

fn consume(owns s: String):
    println(s)

fn main():
    s = String("a")
    call_twice(fn(): consume(s))    #$ error[E3013]: `consume(s)` would hand `main`'s `s` over from inside a function that `main` keeps; give the function its own `s` to hand over, `fn() with owns s: consume(s)` (then it runs once), or hand over a copy, `consume(copy of s)`
