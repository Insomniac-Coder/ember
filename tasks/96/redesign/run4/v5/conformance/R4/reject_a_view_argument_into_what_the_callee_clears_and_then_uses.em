#$ test: compile-fail
#$ rules: BRW-21
#$ help: note(copy of LOG[0])
## design rules: R4, R8, R17
## R4, R8 (g), verdict 53: the compiler works out from `note`'s body that it uses its view parameter
## `s` after `LOG.clear()`, keeps that fact with `note`, and refuses a call that hands `s` a part of
## `LOG`: `s` would read freed text (E3031). Nothing is checked while the program runs.

static mut LOG: Array[String] = []

fn note(s: StrView):
    LOG.clear()
    println(s)

fn main():
    LOG.push(String("hello"))
    note(LOG[0])    #$ error[E3031]: `note` clears `LOG` (line 11) and then uses `s` (line 12), and `LOG[0]` is part of `LOG`; give it a copy: `note(copy of LOG[0])`, or in `note` use `s` before `LOG.clear()`
