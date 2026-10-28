#$ test: compile-fail
#$ rules: OWN-3
#$ help: call it once: remove line 10
## design rules: R12, R1
## R12 (rules.md line 782): "`once fn()` is a function a call uses up"; a second call uses a name already
## used up (E3040, rules.md section 4's line 1188: "a name used after it was handed over or ended"). No
## message is given in the rules; this one follows R18 and verdict 73: each way out compiles here.

fn call_once(owns f: once fn()):
    f()
    f()    #$ error[E3040]: `f` is a `once fn()`, used up by its call on line 9, and is called again on line 10; call it once: remove line 10; or, to run it twice, take a function that may run twice and give it one that hands over a copy each time: write `fn call_once(f: fn()):` in place of line 8 and `call_once(fn(): consume(copy of s))` in place of line 17

fn consume(owns s: String):
    println(s)

fn main():
    s = String("a")
    call_once(fn() owns s: consume(s))
