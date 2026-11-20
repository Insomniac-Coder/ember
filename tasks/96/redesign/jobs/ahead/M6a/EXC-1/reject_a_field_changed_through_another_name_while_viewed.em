# Clause EXC-1.1: "A change of an object's field while a read-only view into that field, made
# through another name that may reach the object, is still used is refused when compiling (`E3021`,
# at the change), and a change that may reshape it ends the view instead (`E3060`, at its later
# use), by `[BRW-1]`'s promise; a change through another name while a `ref mut` name of the field
# is still used is `E3022`."
#$ test: compile-fail
#$ rules: EXC-1, BRW-1, BRW-17, EXC-3
# The other name is a link opened `ref mut`, which may lead to `p`'s object. One function per case;
# no text in the spec for these programs: code only.

class Player:
    name: String
    hp: int = 5

    fn init(owns name: String):
        this.name = name

class Camera:
    target: weakref Player = None

fn read_then_write():
    p = Player(String("ann"))
    cam = Camera()
    cam.target = weakref p
    v = ref p.hp
    if cam.target exists as ref mut t:
        t.hp = 1    #$ error[E3021]
    println(v)

fn view_then_reshape():
    p = Player(String("ann"))
    cam = Camera()
    cam.target = weakref p
    n = p.name
    if cam.target exists as ref mut t:
        t.name = String("bob")
    println(n)    #$ error[E3060]

fn ref_mut_then_write():
    p = Player(String("ann"))
    cam = Camera()
    cam.target = weakref p
    r = ref mut p.hp
    if cam.target exists as ref mut t:
        t.hp = 2    #$ error[E3022]
    r += 1

fn main():
    read_then_write()
    view_then_reshape()
    ref_mut_then_write()
