#$ test: compile-fail
#$ rules: GRM-8a, CLI-19
## `[GRM-8a]` — `shares`, `thread_shared`, `weakref` and `sharedref` after `[` commit to a type argument
## too, so the line is read as a type with that word before it (the word's meaning is the owners'
## milestone's: `E0900` until it is built, `[CLI-19]`), never as an expression.

struct Pt:
    x: int = 1

fn main():
    a = mem.size_of[shares Pt]()
    b = mem.size_of[thread_shared Pt]()
    c = mem.size_of[weakref Pt]()
    d = mem.size_of[sharedref mut Pt]()    #$ error[E0900]: `sharedref mut` before a type is not built by this compiler yet
    println(a, b, c, d)
