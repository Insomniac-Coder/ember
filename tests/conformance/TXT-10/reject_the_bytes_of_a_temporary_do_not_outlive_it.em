#$ test: compile-fail
#$ rules: TXT-10, BRW-8
# `[TXT-10]` — `as_bytes()` borrows the text: the bytes of a `String` made for
# one statement cannot be kept past it.

fn make() -> String:
    return "x"

fn main():
    b = make().as_bytes()   #$ error[E3060]: this temporary is dropped at the end of its statement while it is still borrowed
    println(len(b))
