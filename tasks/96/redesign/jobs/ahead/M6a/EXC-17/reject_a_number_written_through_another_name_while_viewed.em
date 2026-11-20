# Clause EXC-17.2: "While a read-only view of that field is still used, a write through another name
# that may reach it is `E3021` (`[BRW-1]`). Through a `sharedref mut` link, any holder may write it
# at any time on one thread (`[LT-46]`), except while a read-only view of it made through another
# name is still used (`E3021`, the sentence before)"
#$ test: compile-fail
#$ rules: EXC-17, BRW-1, LT-46
# A write through a `weakref` link opened `ref mut`, and through a kept `sharedref mut` link
# (`[BRW-17]`'s `Watcher`). No text in the spec for these: code only.

class Player:
    hp: int = 5

class Camera:
    target: weakref Player = None

class Watcher:
    target: sharedref mut Player

fn main():
    p = Player()
    cam = Camera()
    cam.target = weakref p
    v = ref p.hp
    if cam.target exists as ref mut t:
        t.hp = 1    #$ error[E3021]
    println(v)
    players = [Player()]
    w = Watcher(target=sharedref mut players[0])
    w.target.hp -= 1
    u = ref players[0].hp
    w.target.hp -= 1    #$ error[E3021]
    println(u)
