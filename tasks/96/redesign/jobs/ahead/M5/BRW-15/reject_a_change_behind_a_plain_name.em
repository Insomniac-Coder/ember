# Clause BRW-15.1: "[a plain name] never change[s] what they name or what it leads to (a `Cell`
# field, an object behind a `shares` link, the elements of a `MutSpan`). A change through one is
# `E3023`"
#$ test: compile-fail
#$ rules: BRW-15, FN-1, CELL-1, RC-1
# Cases the spec names but gives no message for: codes only. `r = 99` is Q3 (INDEX.md).

struct Sprite:
    frame: Cell[int]

class Game:
    score: int = 0

struct Enemy:
    hp: int

fn advance(s: Sprite):
    s.frame.set(s.frame.get() + 1)    #$ error[E3023]

fn award(g: shares mut Game):
    g.score += 1    #$ error[E3023]

fn main():
    sprite = Sprite(frame=Cell(0))
    advance(sprite)
    game = shares mut Game()
    award(game)
    enemies = [Enemy(hp=1)]
    boss = enemies[0]
    boss.hp = 2    #$ error[E3023]
    x = 1
    r = ref x
    r = 99    #$ error[E3023]
    println(x)
