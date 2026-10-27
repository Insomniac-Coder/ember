#$ test: compile-fail
#$ rules: R15, R9
## R15: E7010 where chunks may write one place: a list of `shares mut` objects may hold one object twice
## (section 3's message; AN-74).

class Player:
    hp: int = 10
    x: int = 0

fn hurt_all(ref mut players: Array[shares mut Player]):
    @parallel
    for ref mut p in players:    #$ error[E7010]: `players` holds `shares mut Player` objects, and one object may be in the list twice, so two chunks could change it at once; give each object one owner, `players: Array[Player]`, or guard the fields: in `Player` write `lock: Lock = Lock()`, `hp: int guarded by lock = 10` and `x: int guarded by lock = 0`, and put the loop's body inside `with p.lock:`
        p.hp -= 1
        p.x += p.hp & 1

fn main():
    players: Array[shares mut Player] = [shares mut Player(), shares mut Player()]
    hurt_all(ref mut players)
