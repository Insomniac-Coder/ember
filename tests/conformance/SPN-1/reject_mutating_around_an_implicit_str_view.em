#$ test: compile-fail
#$ rules: SPN-1, BRW-1, BRW-2
# The implicit coercion must create exactly the same live shared loan as
# `as_str()`. If it were represented as a conversion, this mutation could
# reallocate `s` while `view` still points into the old buffer.

fn main():
    s: String = String()
    s.push_str("hi")
    view: str = s
    s.push_str("!")  #$ error[E3021]: `s` is borrowed here and mutably borrowed elsewhere
    println(view)
