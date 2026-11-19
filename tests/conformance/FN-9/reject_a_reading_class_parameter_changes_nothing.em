#$ test: compile-fail
#$ rules: FN-9, FN-1, EXC-15
# `[FN-9]` — `p: Player` reads the object, which neither changes nor ends while `p` is used: a method
# that may change its object is not called through a `p` that only reads it (`E3023`, `[EXC-15]`), and
# neither is a field written through it. `p = Player()` is not the caller's object given another
# (that is `ref mut p`).

class Player:
    hp: int

    fn heal():
        this.hp += 1

fn bad_method(p: Player):
    p.heal()    #$ error[E3023]: `p.heal()` changes the caller's player, but `p` only reads it; write `fn bad_method(ref mut p: Player):` and call it `bad_method(ref mut a)`

fn bad_field(p: Player):
    p.hp = 4    #$ error[E3023]: `bad_field` sets `p.hp`, but `p` only reads the caller's player; write `fn bad_field(ref mut p: Player):` and call `bad_field(ref mut a)`

fn main():
    a = Player(1)
    bad_method(a)
    bad_field(a)
