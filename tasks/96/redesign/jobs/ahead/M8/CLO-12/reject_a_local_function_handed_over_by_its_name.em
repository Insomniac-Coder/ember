# Clause CLO-12.1: "A local function (`[GRM-28]`) is a named lambda: it takes the same capture words
# after its brackets (...), and may be passed by its name"; "with `owns` as before every argument for
# an `owns` parameter (`[FN-2a]`): (...) `push_a` is handed over, and `push_a()` after that line is
# `E3040`; `a = thread.spawn(push_a)` is `E3028`"
#$ test: compile-fail
#$ rules: CLO-12, GRM-28, FN-2a
# A function that keeps what it is given stands for `thread.spawn` (M9's). No text in the spec: code
# only.

fn keep(owns f: fn()):
    f()

fn handed_over():
    n = 5
    fn show() copy of n:
        println(n + 1)
    keep(owns show)
    show()    #$ error[E3040]

fn without_the_word():
    n = 1
    fn show() copy of n:
        println(n)
    keep(show)    #$ error[E3028]

fn main():
    handed_over()
    without_the_word()
