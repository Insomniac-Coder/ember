#$ test: compile-fail
#$ rules: R12, R1
#$ help: fn call_twice(f: fn()):
## R12 (rules.md line 648): "`once fn()` is a function a call uses up"; a second call uses a name already
## used up (E3040, rules.md line 945: "a name used after it was handed over or ended"). No message is
## given in the rules; this one follows R18.

fn call_once(owns f: once fn()):
    f()
    f()    #$ error[E3040]: `f` is a `once fn()`, used up by its call on line 9, and is called again on line 10; call it once, or, to run a function twice, take `f: fn()` (`fn call_twice(f: fn()):`) and give it one that hands over a copy each time, `fn(): consume(copy of s)`

fn consume(owns s: String):
    println(s)

fn main():
    s = String("a")
    call_once(fn() with owns s: consume(s))
