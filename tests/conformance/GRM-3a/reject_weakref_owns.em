#$ test: compile-fail
#$ rules: GRM-3a
## `[GRM-3a]` — `weakref owns T` is `E0100`: a value held `owns` has no header for a link to test.

class Mesh:
    n: int = 0

fn main():
    last: weakref owns Mesh = None    #$ error[E0100]: a value held `owns` has no header for a link to test
                                      #$ help: a link names a class object or a value held `shares` or `thread_shared`
                                      #$ help: `last: weakref Mesh = None`
    println(1)
