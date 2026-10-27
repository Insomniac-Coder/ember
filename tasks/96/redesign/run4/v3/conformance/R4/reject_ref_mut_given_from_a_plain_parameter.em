#$ test: compile-fail
#$ rules: R4, R5
#$ help: fn revive(ref mut p: Player):
## R4: `ref mut` before a plain parameter is refused: `p` only reads the caller's player, so this
## function may not hand it on to be changed (rules section 3, E3027).

class Player:
    hp: int = 0

fn heal(ref mut p: Player):
    p.hp = 100

fn revive(p: Player):
    heal(ref mut p)    #$ error[E3027]: `p` only reads the caller's player, so it cannot be given to `heal` as `ref mut`; take it as `ref mut p: Player` on `revive`'s first line: `fn revive(ref mut p: Player):`, and call it `revive(ref mut hero)`

fn main():
    hero = Player()
    revive(hero)
    println(hero.hp)
