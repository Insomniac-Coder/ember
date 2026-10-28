#$ test: compile-fail
#$ rules: WK-16
#$ help: cam.target = weakref player
## design rules: R11
## R11: "a `weakref` of a part, a field or an element stored inline, is refused (E2119)": `player.pos`
## is a `Pos` stored inside the player, with no header of its own a link could test. The field form of
## rules section 3's E2119 row (the element form is `reject_a_weakref_of_a_struct_stored_inline_in_a_list.em`);
## its message, in this program's names, with the lines it replaces.

struct Pos:
    x: int = 0

class Player:
    pos: Pos = Pos(x=3)

class Camera:
    target: weakref Player = None

fn main():
    player = Player()
    cam = Camera()
    at = weakref player.pos             #$ error[E2119]: `player.pos` is a `Pos` stored inside the player, and a `weakref` link needs an object that lives on its own; link to the player: write `cam.target = weakref player` in place of line 21, `if cam.target exists as p:` in place of line 22 and `println(p.pos.x)` in place of line 23
    if at exists as q:
        println(q.x)
