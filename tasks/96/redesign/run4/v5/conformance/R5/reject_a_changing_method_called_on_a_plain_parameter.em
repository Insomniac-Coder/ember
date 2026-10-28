#$ test: compile-fail
#$ rules: BRW-15
#$ help: fn revive(ref mut p: Player):
## design rules: R5, R4
## R5, R4: a plain parameter never changes anything, so a method that changes its object is refused on
## it, as on a plain loop name (AN-18's `g.reset()`, E3023).

class Player:
    hp: int = 0

    fn heal():
        this.hp = 100

fn revive(p: Player):
    p.heal()    #$ error[E3023]: `heal` changes the player (`this.hp = 100`, line 11), and `p` only reads the caller's player; write `fn revive(ref mut p: Player):` and call it `revive(ref mut hero)`

fn main():
    hero = Player()
    revive(hero)
    println(hero.hp)
