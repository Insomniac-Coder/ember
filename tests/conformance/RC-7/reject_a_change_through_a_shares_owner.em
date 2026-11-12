#$ test: compile-fail
#$ rules: RC-7, OWN-13
#$ help: to change it through `s`, write `s = shares mut p` in place of line 12
# `[RC-7]` — a value made shared is changed by its owners that may change it; a `shares` owner only
# reads it (`E3023`).

class Player:
    hp: int = 100

fn main():
    p = Player()
    s = shares p
    s.hp = 5                          #$ error[E3023]: `s.hp` changes the player, but `s` only reads the player it shares
    println(p.hp)
