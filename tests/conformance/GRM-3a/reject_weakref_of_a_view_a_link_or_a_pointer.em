#$ test: compile-fail
#$ rules: GRM-3a
## `[GRM-3a]` — `weakref ref T`, `weakref ref mut T`, `weakref sharedref mut T`, `weakref weakref T`
## and `weakref *T` are `E0100`: a link names the value itself, not a view, a link or a pointer to it.

class Player:
    hp: int = 0

fn main():
    a: weakref ref Player = None    #$ error[E0100]: a link names the value itself, not a view, a link or a pointer to it
                                    #$ help: write `a: weakref Player = None`
    b: weakref ref mut Player = None    #$ error[E0100]: a link names the value itself, not a view, a link or a pointer to it
    c: weakref sharedref mut Player = None    #$ error[E0100]: a link names the value itself, not a view, a link or a pointer to it
    d: weakref weakref Player = None    #$ error[E0100]: a link names the value itself, not a view, a link or a pointer to it
    e: weakref *Player = None    #$ error[E0100]: a link names the value itself, not a view, a link or a pointer to it
    println(1)
