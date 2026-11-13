#$ test: compile-fail
#$ rules: FN-2, BRW-14
# `[FN-2]` — a parameter with no word reads: there is no by-copy mode, so changing it is refused.

fn grow(s: String):
    s.push_str("x")                 #$ error[E3023]: `s.push_str("x")` changes the caller's text, but `s` only reads it; write `fn grow(ref mut s: String):` and call it `grow(ref mut s)`

fn main():
    s = String("ab")
    grow(s)
