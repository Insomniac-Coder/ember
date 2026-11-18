#$ test: compile-fail
#$ rules: FN-2a, BRW-15
#$ help: take it as `ref mut p: Player` on `revive`'s first line
#$ help: to change that player through `p`, write `for ref mut p in players:`
# `[FN-2a]` — `ref mut` before something this function may not change (a plain parameter, a loop
# name over something it only reads) is `E3027`, in the program's own names.

struct Player:
    hp: int

fn heal(ref mut p: Player):
    p.hp += 1

fn revive(p: Player):
    heal(ref mut p)    #$ error[E3027]: `p` only reads the caller's player, so it cannot be given to `heal` as `ref mut`

fn main():
    players = [Player(hp=1)]
    for p in players:
        heal(ref mut p)    #$ error[E3027]: `p` only reads the player in `players`, so it cannot be given to `heal` as `ref mut`
    revive(players[0])
