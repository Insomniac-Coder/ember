# Clause BCK-7.1: "A place reached through a link (`shares`, `shares mut`, `thread_shared`,
# `weakref`, `sharedref mut`) is in another object, and two paths to such places follow
# `[BCK-3]`'s rule for links."
#$ test: compile-fail
#$ rules: BCK-7, BCK-3, EXC-3
# `t` and `u` are opened from two links of one type, so they may name one player: reading `u.hp`
# while `t`, a `ref mut` name, is still used is `E3021`. No text in the spec: code only.

class Player:
    hp: int = 5

class Camera:
    target: weakref Player = None

fn main():
    players = [Player(), Player()]
    a = Camera()
    b = Camera()
    a.target = weakref players[0]
    b.target = weakref players[1]
    if a.target exists as ref mut t:
        if b.target exists as u:
            t.hp = u.hp + 1    #$ error[E3021]
            println(t.hp)
