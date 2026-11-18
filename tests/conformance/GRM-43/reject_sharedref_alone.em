#$ test: compile-fail
#$ rules: GRM-43
## `[GRM-43]` — `sharedref` is always followed by `mut`, before a value and in a type alike: alone it is
## `E0114`, whose help says to write `ref` for a link that only reads.

class Player:
    hp: int = 0

fn main():
    p = Player()
    t = sharedref p    #$ error[E0114]: `sharedref` is always followed by `mut`
                       #$ help: write `target: ref Player`
    u: sharedref Player = p    #$ error[E0114]: `sharedref` is always followed by `mut`
    println(1)
