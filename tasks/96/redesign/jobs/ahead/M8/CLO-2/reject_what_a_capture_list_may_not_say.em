# Clause CLO-2.3: "`ref mut` is not a capture word (`E0120`: ...); a captured name with no word after
# one with a word is `E0116` (...); `with` before the capture words is `E0100` (...); `owned fn` is
# `E1010` and `owns fn` `E0109`"
#$ test: compile-fail
#$ rules: CLO-2, GRM-42
# The spec's `E0120` and capture-list `E0116` messages (the existing GRM-42 tests claim shorter or
# other texts: §2); `owns fn`, whose message the spec describes but does not give, is code only.
# `with` and `owned fn` are tested in `tests/conformance` already.

fn main():
    x = 1
    name = String("a")
    score = 2
    f = fn() ref mut x: x += 1    #$ error[E0120]: remove `ref mut x`: without a word the function already uses the real `x`, and may change it: `fn(): x += 1`
    g = fn() copy of name, score: println(name, score)    #$ error[E0116]: `copy of` here belongs to `name` only; say what the function does with `score`: `fn() copy of name, copy of score:`, or leave `score` out to use the real `score`
    h = owns fn(): println(name)    #$ error[E0109]
    f()
    g()
    h()
