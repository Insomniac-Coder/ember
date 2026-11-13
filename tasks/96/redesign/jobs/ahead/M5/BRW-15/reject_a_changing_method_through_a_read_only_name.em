# Clause BRW-15.2: "A method that may change its object, called through a name that only reads it,
# is `E3023` too (`[EXC-15]`)."
#$ test: compile-fail
#$ rules: BRW-15, EXC-15, BRW-10
# The first message is `[EXC-15]`'s own example, with the file's line numbers; the second case has
# no message text in the spec: code only.

class Player:
    hp: int = 0
    fn heal():
        this.hp += 10

fn heal_one(p: Player):
    p.heal()    #$ error[E3023]

fn main():
    players = [Player(), Player()]
    for p in players:
        p.heal()    #$ error[E3023]: `heal` changes the player (`this.hp += 10`, its line 11), and `p` only reads it; write `for ref mut p in players:` in place of line 18
    heal_one(players[0])
