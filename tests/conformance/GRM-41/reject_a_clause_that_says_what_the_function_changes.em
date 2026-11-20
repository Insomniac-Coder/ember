#$ test: compile-fail
#$ rules: GRM-41, EFF-4
## `[GRM-41]` — nothing else follows the brackets: a clause that says what the function changes or reads
## (`changes FRAME`, `reads renderer`) is `E0100`, whose message gives the first line without it.

fn tick() changes FRAME:    #$ error[E0100]: `changes FRAME` is not Ember: the compiler works out what `tick` changes from its body; write `fn tick():`
    pass

fn draw(n: int) -> int reads renderer:    #$ error[E0100]: `reads renderer` is not Ember: the compiler works out what `draw` reads from its body; write `fn draw(n: int) -> int:`
    return n

fn main():
    tick()
    println(draw(1))
