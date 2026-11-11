#$ test: compile-fail
#$ rules: FN-2, BRW-14
# `[FN-2]` — a parameter with no word reads: there is no by-copy mode, so changing it is refused.

fn grow(s: String):
    s.push_str("x")                 #$ error[E3023]: cannot mutate borrowed parameter `s`

fn main():
    s = String("ab")
    grow(s)
