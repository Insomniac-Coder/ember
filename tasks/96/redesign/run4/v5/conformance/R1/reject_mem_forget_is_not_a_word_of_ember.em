#$ test: compile-fail
#$ rules: MOD-3
#$ help: to end `a` here with its clean-up write `mem.drop(a)` in place of line 21; to give it to a C function that keeps it, hand it to a parameter its declaration says it takes (`owns`), or, inside `unsafe:`, give it `a.into_raw()`
## design rules: R1
## R1: a value always ends with its clean-up: when its last owner ends, its `drop` runs, and nothing
## ends an owner without it. `mem.forget` is not Ember (E1010; verdict 58: it left a shared count too
## high for good, and was the cheap way to roll a count over). Handing an object to C goes to an `owns`
## parameter its declaration says it takes, or by `into_raw()` inside `unsafe:` (verdict 91; verdict 58's
## corrected record: the `Retained` token gives no object away). The nearest program that is allowed is
## `accept_a_value_ended_now_runs_its_clean_up`.

import std.mem

class Res:
    name: String

    fn drop():
        println("end", this.name)

fn main():
    a = Res(name=String("one"))
    mem.forget(a)              #$ error[E1010]: `mem.forget` is gone: a value always ends with its clean-up
    println("after")
