#$ test: compile-fail
#$ rules: R12
#$ help: call_twice(fn(): consume(copy of s))
## R12 (rules.md lines 773-775): "a function value that gives a capture away is" a `once fn()`; passed
## where it may run twice it is refused (E3013). AN-59 part 6; rules.md section 3's line 1045 message,
## word for word (corrected in v4: its earlier first way out, `owns f: once fn()` in a `call_twice` that
## calls twice, was refused itself; verdict 73).

fn call_twice(f: fn()):
    f()
    f()

fn consume(owns s: String):
    println(s)

fn main():
    s = String("a")
    call_twice(fn() with owns s: consume(s))    #$ error[E3013]: this function gives `s` away, so it can run only once, and `call_twice` runs it twice; hand over a copy each time instead: `call_twice(fn(): consume(copy of s))`
