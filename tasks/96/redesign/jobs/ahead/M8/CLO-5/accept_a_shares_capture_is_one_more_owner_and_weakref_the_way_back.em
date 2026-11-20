# Clause CLO-5.1: "A function value that captures an object with `shares` or `thread_shared` holds one
# more owner of it"; "the link back is captured `weakref` (`fn() weakref ui:`), or the object is given
# to the function as a parameter"
#$ test: run-pass
#$ rules: CLO-5, CLO-2, WK-1
#$ stdout: 5
#$ stdout: 9
#$ stdout: 1
# The first player lives on in `g`'s owner after `p` is given another; a handler stored in the `Ui`
# it links back to captures it `weakref`.

class Player:
    hp: int = 5

class Ui:
    handlers: Array[fn()] = []

fn count_handlers(w: weakref Ui):
    if w exists as u:
        println(u.handlers.len())

fn main():
    p = shares Player()
    g = fn() shares p => p.hp
    p = shares Player(hp=9)
    println(g())
    println(p.hp)
    ui = shares mut Ui()
    ui.handlers.push(owns fn() weakref ui: count_handlers(ui))
    for ref mut h in ui.handlers:
        h()
