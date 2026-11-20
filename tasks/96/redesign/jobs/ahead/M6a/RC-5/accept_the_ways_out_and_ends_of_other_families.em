# Clause RC-5.1: "use `p` before line 12, or open the link again after it: write `if cam.target
# exists as q:` above line 13, indent line 13 under it and use `q` there"; "No line or call in the
# view's life may end an object of that type's class family"
#$ test: run-pass
#$ rules: RC-5, EXC-3, DIA-25
#$ stdout: 5
#$ stdout: gone
#$ stdout: 5
# The message's two ways out compile; and clearing a list of another family's objects (`Rock`
# shares no root base class with `Player`) does not end what `p` reads.

class Player:
    hp: int = 5

class Rock:
    weight: int = 9

class Camera:
    target: weakref Player = None

fn remove_all(ref mut ps: Array[Player]):
    ps.clear()

fn main():
    players = [Player()]
    rocks = [Rock()]
    cam = Camera()
    cam.target = weakref players[0]
    if cam.target exists as p:
        rocks.clear()
        println(p.hp)
    remove_all(ref mut players)
    if cam.target exists as q:
        println(q.hp)
    else:
        println("gone")
    players.push(owns Player())
    cam.target = weakref players[0]
    if cam.target exists as r:
        println(r.hp)
