#$ test: compile-fail
#$ rules: CTL-10
# `[CTL-10]` — different types in different arms are `E2230`, naming each
# arm's; the name is still declared, so its later uses add no error. A name
# declared in only some arms, or under an `if` with no `else`, stays in its
# arm.

fn main():
    n = 1
    if n > 0:
        v = 1               #$ error[E2230]: `v` is assigned in every branch, but at different types
    else:
        v = "one"
    println(v)
    if n > 0:
        w = 2
    println(w)              #$ error[E1010]: cannot find `w` in this scope
