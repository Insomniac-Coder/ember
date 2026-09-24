#$ test: compile-fail
#$ rules: TXT-10, BRW-8
# `[TXT-10]` — `trim` returns a view of the text it trims, so the text must
# outlive it: a `String` made for one statement cannot be trimmed and kept.

fn make() -> String:
    return "  hi  "

fn main():
    v = make().trim()       #$ error[E3060]: this temporary is dropped at the end of its statement while it is still borrowed
    println(v)
