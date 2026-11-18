# Clause BRW-17.6: "Removing, clearing or replacing still ends every view of what it may end."
#$ test: compile-fail
#$ rules: BRW-17, LT-46, BCK-3, RC-5
# A kept view of a class object, which an add leaves alone, ended by each. Codes only.

class Player:
    hp: int = 10

struct Watcher:
    target: sharedref mut Player

fn removed():
    players = [Player(), Player()]
    w = Watcher(target=sharedref mut players[0])
    players.remove(0)
    w.target.hp -= 1    #$ error[E3060]

fn cleared():
    players = [Player()]
    w = Watcher(target=sharedref mut players[0])
    players.clear()
    w.target.hp -= 1    #$ error[E3060]

fn replaced():
    players = [Player()]
    w = Watcher(target=sharedref mut players[0])
    players[0] = Player(hp=1)
    w.target.hp -= 1    #$ error[E3060]

fn main():
    removed()
    cleared()
    replaced()
