#$ test: compile-fail
#$ rules: PAN-2
## `[PAN-2]` — formatting a panic message allocates for an f-string, so one in `@noalloc` code is
## `E4001`.

@noalloc
fn check(free: int):
    if free < 1:
        panic(f"{free} left")    #$ error[E4001]: reaches an allocation

fn main():
    check(5)
