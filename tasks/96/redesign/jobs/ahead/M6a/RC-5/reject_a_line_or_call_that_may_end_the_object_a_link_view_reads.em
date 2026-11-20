# Clause RC-5.1: "No line or call in the view's life may end an object of that type's class family
# (`[EXC-3]`: an `Array[Entity]` cleared ends a `Player` derived from `Entity`), held directly or
# through anything that owns it, all the way down (a list of teams cleared ends their players)";
# "After such a line or call the view is not used (`E3060`"
#$ test: compile-fail
#$ rules: RC-5, BCK-7, EXC-3, EFF-1
# The first message is the spec's, with this file's line numbers. A list of the base class cleared,
# and a list of teams cleared, end a `Player` too: code only. The end of an object whose `drop`
# ends one is DRP-8's and the hand-over to a thread M9's: not here.

open class Entity:
    hp: int = 5

    fn init():
        pass

class Player(Entity):
    fn init():
        super.init()

class Team:
    players: Array[Player] = []

class Camera:
    target: weakref Player = None

fn remove_all(ref mut ps: Array[Player]):
    ps.clear()

fn by_call():
    players = [Player()]
    cam = Camera()
    cam.target = weakref players[0]
    if cam.target exists as p:
        println(p.hp)
        remove_all(ref mut players)
        println(p.hp)    #$ error[E3060]: `remove_all(ref mut players)` (line 36) may destroy the player `p` reads (`if cam.target exists as p:`, line 34), and `p` is used on line 37; use `p` before line 36, or open the link again after it: write `if cam.target exists as q:` above line 37, indent line 37 under it and use `q` there

fn by_family(ref mut things: Array[Entity], cam: Camera):
    if cam.target exists as p:
        things.clear()
        println(p.hp)    #$ error[E3060]

fn by_owner(ref mut teams: Array[Team], cam: Camera):
    if cam.target exists as p:
        teams.clear()
        println(p.hp)    #$ error[E3060]

fn main():
    by_call()
    ents: Array[Entity] = []
    teams: Array[Team] = []
    cam = Camera()
    by_family(ref mut ents, cam)
    by_owner(ref mut teams, cam)
