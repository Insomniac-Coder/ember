#$ test: compile-fail
#$ rules: R12
#$ help: fn call_once(owns f: once fn()):
## R12 (rules.md lines 648-650): "a function value that gives a capture away is" a `once fn()`; passed
## where it may run twice it is refused (E3013). AN-59 part 6; rules.md section 3's line 865 message,
## word for word.

fn call_twice(f: fn()):
    f()
    f()

fn consume(owns s: String):
    println(s)

fn main():
    s = String("a")
    call_twice(fn() with owns s: consume(s))    #$ error[E3013]: this function gives `s` away, so it can run only once, and `call_twice` may run it twice; if `call_twice` runs it once, declare the parameter `owns f: once fn()`; otherwise hand over a copy each time: `fn(): consume(copy of s)`
