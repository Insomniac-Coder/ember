# Clause BRW-1.2: "another name that may name the same place, a link of the same type included, is
# refused, `E3022` to change it and `E3021` to read it"
#$ test: compile-fail
#$ rules: BRW-1, BRW-17, BCK-3, BCK-7, GRM-44
# `p` is a `ref mut` parameter, alone while it is used; `t`, opened from a `weakref Player` link, may
# be the same player (a player aimed at itself). Codes only. The opening reads the link, a field of
# `p`, through `p`; the refusal is at the read or the change through `t`.

class Player:
    hp: int = 10
    target: weakref Player = None

fn read_through_a_link(ref mut p: Player):
    if p.target exists as t:
        println(t.hp)    #$ error[E3021]
    p.hp -= 1

fn changed_through_a_link(ref mut p: Player):
    if p.target exists as ref mut t:
        t.hp -= 1    #$ error[E3022]
    p.hp -= 1

fn main():
    a = Player()
    b = Player()
    a.target = weakref b
    read_through_a_link(ref mut a)
    changed_through_a_link(ref mut a)
    println(a.hp, b.hp)
