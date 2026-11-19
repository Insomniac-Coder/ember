#$ test: run-pass
#$ rules: OWN-9, FN-2a
#$ stdout: 2 1
# `[OWN-9]` — a value made in the call with a word that makes an owner or a link needs no `owns`.

class Game:
    level: int = 1

fn setup(owns g: shares mut Game) -> int:
    return g.level + 1

fn watch(owns w: weakref Game) -> int:
    if w exists as g:
        return g.level
    return 0

fn main():
    game = shares mut Game()
    println(setup(shares mut game), watch(weakref game))
