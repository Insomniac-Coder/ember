# Clause BRW-14.4: "It is made by the refused line itself where that line is a plain `=` at the top
# level of the body and no line above it reads the parameter (`new_line = line.trim()`), and
# otherwise by a new first line of the body (`new_n = n`; `new_s = copy of s` for a value that is
# neither `Copy` nor a view, offered only where `copy of` applies), never inside a loop or a branch,
# where it would be made again each time"
#$ test: compile-fail
#$ rules: BRW-14, DIA-25, FN-2
# One function per case, with the whole message. Its noun is Q5 (i)'s (INDEX.md), the same in its
# three places: "text" for a `StrView`, "number" for an `int`, "list" for an `Array`, and a type's
# own name in lower case. `Handle` has no `Clone`, so no copy is offered: Q5 (ii), the callee takes
# it over (`[FN-2]`), each call named; only that part of the text is pinned.

struct Handle:
    id: int
    data: Array[int]

fn shout(s: StrView) -> int:
    println("start")
    s = s.trim()    #$ error[E3023]: `s` is the text the caller passed and stays that text for the whole call; for a text of your own write `new_s = s.trim()` in place of line 19, and `new_s` for `s` in line 20
    return s.len()

fn twice(n: int) -> int:
    println(n)
    n = n * 2    #$ error[E3023]: `n` is the number the caller passed and stays that number for the whole call; for a number of your own write `new_n = n` above line 23, as the first line of `twice`'s body, and `new_n` for `n` in lines 23, 24 and 25
    return n

fn clamp(n: int) -> int:
    if n > 10:
        n = 10    #$ error[E3023]: `n` is the number the caller passed and stays that number for the whole call; for a number of your own write `new_n = n` above line 28, as the first line of `clamp`'s body, and `new_n` for `n` in lines 28, 29 and 30
    return n

fn grow(names: Array[String]) -> int:
    println(names.len())
    names = [String("x")]    #$ error[E3023]: `names` is the list the caller passed and stays that list for the whole call; for a list of your own write `new_names = copy of names` above line 33, as the first line of `grow`'s body, and `new_names` for `names` in lines 33, 34 and 35
    return names.len()

fn reopen(h: Handle) -> int:
    println(h.id)
    h = Handle(id=2, data=[])    #$ error[E3023]: `h` is the handle the caller passed and stays that handle for the whole call; for a handle of your own take it over: write `fn reopen(owns h: Handle) -> int:` and call it `reopen(owns first)` on line 44
    return h.id

fn main():
    first = Handle(id=1, data=[1])
    println(shout(" a "), twice(2), clamp(20), grow([String("a")]), reopen(first))
