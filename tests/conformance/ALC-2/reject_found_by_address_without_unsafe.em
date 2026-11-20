#$ test: compile-fail
#$ rules: ALC-2, TYP-43
## `[ALC-2]` (verdict 107) — implementing `FoundByAddress` is a promise the compiler cannot check, so
## it is `unsafe`: a plain `extend` is `E3100`.

from std.mem import FoundByAddress

struct Pool:
    pass

extend Pool implements FoundByAddress:    #$ error[E3100]: implementing `std.mem.FoundByAddress` is `unsafe`
    pass

fn main():
    pass
