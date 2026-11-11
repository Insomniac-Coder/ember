#$ test: compile-fail
#$ rules: MOD-7
## `[MOD-7]` — `pub(read)` is a field's visibility: on a struct, a function or any other item it is
## `E1051`.

pub(read) struct Point:    #$ error[E1051]: `read` visibility is valid on fields only
    x: int

fn main():
    p = Point(x=1)
    println(p.x)
