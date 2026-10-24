# Attack on names-follow-values rule 1: a @derive(Copy) struct is copied, not named.
# Python (py_attack_output.txt): copy_struct_name 2.0. Ember today accepts this and prints 1.0.
@derive(Copy)
struct Vec2:
    x: f64
    y: f64

struct Player:
    pos: Vec2

fn main():
    p = Player(pos=Vec2(x=1.0, y=2.0))
    pos = p.pos
    pos.x += 1.0
    println(p.pos.x)          # Python 2.0
