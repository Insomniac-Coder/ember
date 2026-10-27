#$ test: compile-fail
#$ rules: CLO-6
#$ help: call_twice(fn(): consume(copy of s))
## design rules: R12
## R12 (rules.md lines 782-784): "a function value that gives a capture away is" a `once fn()`; passed
## where it may run twice it is refused (E3013). AN-59 part 6; rules.md section 3's line 1096 second
## message, word for word. The nearest allowed program is
## accept_call_once_holds_an_owns_once_fn_and_call_twice_hands_over_copies.

fn call_twice(f: fn()):
    f()
    f()

fn consume(owns s: String):
    println(s)

fn main():
    s = String("a")
    call_twice(fn() owns s: consume(s))    #$ error[E3013]: this function gives `s` away, so it can run only once, and `call_twice` runs it twice; hand over a copy each time instead: `call_twice(fn(): consume(copy of s))`
