# Clause BRW-4.2: "What two different paths may both reach through links is `[BRW-17]`'s."
#$ test: compile-fail
#$ rules: BRW-4, BRW-17, BCK-3, BCK-7, RC-1
# `left` and `right` are different fields, but each is a link to a `Player`, and two paths of one type
# through a link may reach one object: two `ref mut` names of its `hp` are refused. Here they are one
# player. Codes only.

class Player:
    hp: int = 10

class Duel:
    left: shares mut Player
    right: shares mut Player

fn main():
    p = shares mut Player()
    d = Duel(left=shares mut p, right=shares mut p)
    a = ref mut d.left.hp
    b = ref mut d.right.hp    #$ error[E3022]
    a += 1
    b += 1
