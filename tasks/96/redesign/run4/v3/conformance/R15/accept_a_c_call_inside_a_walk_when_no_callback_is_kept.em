#$ test: compile-pass
#$ rules: R15, R12, R8
## R12: "With no kept callback ... a foreign call changes nothing": a program that hands C no callback
## to keep may call an unmarked C function inside a walk.

class Game:
    names: Array[String] = []

unsafe extern "C":
    safe fn poll_events()

fn main():
    game = Game(names=[String("ann"), String("bo")])
    for name in game.names:
        println(name)
        poll_events()
