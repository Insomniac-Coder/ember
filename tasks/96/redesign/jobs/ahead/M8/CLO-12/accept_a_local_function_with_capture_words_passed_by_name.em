# Clause CLO-12.1: "it takes the same capture words after its brackets (`fn push_a() thread_shared
# log:`), follows `[CLO-1]` to `[CLO-6]`, and may be passed by its name, so a long lambda may be
# written inside the call (...) or as a local function"
#$ test: run-pass
#$ rules: CLO-12, GRM-28, CLO-2
#$ stdout: 6
#$ stdout: 6
# The same function written as a local function with a capture word and passed by its name, and
# written inside the call.

fn keep(owns f: fn()):
    f()

fn main():
    n = 5
    fn show() copy of n:
        println(n + 1)
    keep(owns show)
    keep(owns fn() copy of n: println(n + 1))
