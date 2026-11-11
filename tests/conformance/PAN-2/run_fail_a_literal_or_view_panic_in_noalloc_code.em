#$ test: run-fail
#$ rules: PAN-2
#$ profiles: debug, development, release
#$ panics: one left
## `[PAN-2]` — formatting a panic message allocates only for an f-string; `panic("literal")` and
## `panic(some_str)` are `@noalloc`.

@noalloc
fn check(free: int, reason: StrView):
    if free > 1:
        return
    if free == 1:
        panic(reason)
    panic("no room")

fn main():
    check(5, "unused")
    check(1, "one left")
    println("not reached")
