#$ test: compile-fail
#$ rules: ALC-6, ARN-4
## `[ALC-6]` — `Alloc` is the effect of `ember_alloc`, so of every `allocate` that reaches it: a
## growing `Arena`'s does when it grows (`[ARN-4]`), so a line that makes a value in one is refused in
## `@noalloc` code (`E4001`).

struct Cmd:
    id: int

@noalloc
fn make(ref mut level: Arena, id: int) -> owns Cmd using Arena:
    return owns Cmd(id=id) using level    #$ error[E4001]: reaches an allocation

fn main():
    level = Arena.with_capacity(64)
    c = make(ref mut level, 1)
    println(c.id)
