# Clause BRW-14.4: "It is made by the refused line itself where that line is a plain `=` at the top
# level of the body and no line above it reads the parameter (`new_line = line.trim()`), and
# otherwise by a new first line of the body (`new_n = n`; `new_s = copy of s` for a value that is
# neither `Copy` nor a view, offered only where `copy of` applies), never inside a loop or a branch,
# where it would be made again each time"
#$ test: compile-fail
#$ rules: BRW-14, DIA-25
# One function per case. The nouns of a list parameter are not given (Q5): only the line is pinned
# there. A parameter whose type has no `Clone` is not tested (Q5: what is offered is not given).

fn shout(s: StrView) -> int:
    println("start")
    s = s.trim()    #$ error[E3023]: write `new_s = s.trim()` in place of line 13
    return s.len()

fn twice(n: int) -> int:
    println(n)
    n = n * 2    #$ error[E3023]: write `new_n = n` above line 17, as the first line of `twice`'s body
    return n

fn clamp(n: int) -> int:
    if n > 10:
        n = 10    #$ error[E3023]: write `new_n = n` above line 22, as the first line of `clamp`'s body
    return n

fn grow(names: Array[String]) -> int:
    println(names.len())
    names = [String("x")]    #$ error[E3023]: write `new_names = copy of names` above line 27, as the first line of `grow`'s body
    return names.len()

fn main():
    println(shout(" a "), twice(2), clamp(20), grow([String("a")]))
