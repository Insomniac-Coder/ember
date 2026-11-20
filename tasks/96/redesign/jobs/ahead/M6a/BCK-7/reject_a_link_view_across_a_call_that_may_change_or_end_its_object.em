# Clause BCK-7.2: "A view of an object reached through a link lives only while no line and no call
# in its life may end an object of its class family (...) (`[RC-5]` point 1, `[EXC-3]`; `E3060`),
# and across a call that may change such an object a read-only view of one does not live
# (`E3021`)"
#$ test: compile-fail
#$ rules: BCK-7, RC-5, BRW-1
# `heal_all` changes players (a number each), `drop_all` ends them. No text in the spec for these
# programs: code only. `E3021` stands at the call that changes, `E3060` at the use after the end.

class Player:
    hp: int = 5

class Camera:
    target: weakref Player = None

fn heal_all(ref mut ps: Array[Player]):
    for ref mut p in ps:
        p.hp += 1

fn drop_all(ref mut ps: Array[Player]):
    ps.clear()

fn main():
    players = [Player()]
    cam = Camera()
    cam.target = weakref players[0]
    if cam.target exists as p:
        heal_all(ref mut players)    #$ error[E3021]
        println(p.hp)
    if cam.target exists as q:
        drop_all(ref mut players)
        println(q.hp)    #$ error[E3060]
