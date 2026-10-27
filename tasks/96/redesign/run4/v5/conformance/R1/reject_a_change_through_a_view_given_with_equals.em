#$ test: compile-fail
#$ rules: BRW-15
#$ help: to change it through `cur`, write `cur = ref mut p`
## design rules: R1, R5
## R1, R5: `cur = p`, where `p` is a view (here a `ref mut` parameter), makes `cur` a read-only view,
## so a change through `cur` is refused (E3023) with the word that makes it a changing view.

class Player:
    hp: int = 10

fn hurt(ref mut p: Player):
    cur = p
    cur.hp -= 1                #$ error[E3023]: `cur.hp -= 1` changes the player `p` names, but `cur` only reads it (`cur = p`, line 11)

fn main():
    player = Player()
    hurt(ref mut player)
    println(player.hp)
