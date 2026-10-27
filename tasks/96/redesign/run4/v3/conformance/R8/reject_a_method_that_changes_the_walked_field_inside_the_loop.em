#$ test: compile-fail
#$ rules: R8, R5
#$ help: joined: Array[Player] = []
## R8 (c): a list may not change while a loop walks it through a call that may change it by its first
## line: `add_player` says `changes players`. AN-17, point 6.

class Player:
    name: String
    hp: int

class Game:
    players: Array[Player]
    log: Array[String]

    fn add_player(owns p: Player) changes players:
        this.players.push(p)

    fn tick() changes players:
        for ref mut p in this.players:
            p.hp -= 1
            if p.hp <= 0:
                this.add_player(Player(name="imp", hp=3))    #$ error[E3020]: `this.add_player(...)` (line 22) changes `this.players`, which this loop walks (line 19); collect the new players in a list of your own: write `joined: Array[Player] = []` before the loop, `joined.push(Player(name="imp", hp=3))` in place of line 22, and `this.players.extend(joined)` after the loop

fn main():
    g = Game(players=[Player(name="orc", hp=1)], log=[])
    g.tick()
