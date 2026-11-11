#$ test: compile-pass
#$ rules: PAN-2
## `[PAN-2]` — `panic("literal")` and `panic(some_str)` are `@noalloc`.

@noalloc
fn check(free: int, reason: StrView):
    if free > 1:
        return
    if free == 1:
        panic(reason)
    panic("no room")

fn main():
    check(5, "unused")
