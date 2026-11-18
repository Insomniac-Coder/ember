#$ test: compile-fail
#$ rules: GRM-3a
## `[GRM-3a]` — a link does not say how its target's owners hold it: `weakref shares T` and
## `weakref shares mut T` are `E0100`, whose message gives the line to write.

class Player:
    hp: int = 0

fn main():
    scout: weakref shares Player = None    #$ error[E0100]: a link does not say how its target's owners hold it
                                           #$ help: write `scout: weakref Player = None`
    lead: weakref shares mut Player = None    #$ error[E0100]: a link does not say how its target's owners hold it
    println(1)
