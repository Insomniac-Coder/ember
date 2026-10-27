#$ test: run-pass
#$ rules: R4
#$ stdout: 100
## R4: a `ref mut` parameter may be handed on to another `ref mut` parameter, with the word at each
## call; the change reaches `main`'s player.

class Player:
    hp: int = 0

fn heal(ref mut p: Player):
    p.hp = 100

fn revive(ref mut p: Player):
    heal(ref mut p)

fn main():
    hero = Player()
    revive(ref mut hero)
    println(hero.hp)
