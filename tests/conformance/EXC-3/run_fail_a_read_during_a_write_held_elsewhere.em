#$ test: run-fail
#$ rules: EXC-3, EXC-2, EXC-15, EXC-19
#$ profiles: debug, development, release
#$ panics: exclusivity violation: read access to Player.items while a write access to Player.items is active
# `[EXC-3]` — a check is removed only when nothing held anywhere could make it
# fail. `update` holds its object for a call that runs other code, so the read
# in `score` keeps its check, and reading the same player's list panics.

class Player:
    items: Array[int]

    fn init():
        this.items = [1]

    fn update(all: Array[Player]):
        this.items.push(owns 0)
        println(score(all))

fn score(all: Array[Player]) -> int:
    total = 0
    for p in all:
        total = total + len(p.items)
    return total

fn main():
    p = Player()
    all: Array[Player] = []
    all.push(owns p)
    p.update(all)
