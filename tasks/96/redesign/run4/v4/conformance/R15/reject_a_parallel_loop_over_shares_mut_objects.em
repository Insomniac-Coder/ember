#$ test: compile-fail
#$ rules: R15, R9
## R15: E7010 where chunks may write one place: a list of `shares mut` objects may hold one object twice
## (section 3's message, with its import line, check-V6's fix; and the lines of its first way out named,
## R18; AN-74). `hurt_all` and `main` change the players through `shares mut` links, so they say
## `changes Player` (R5, verdicts 63, 64).

class Player:
    hp: int = 10
    x: int = 0

fn hurt_all(ref mut players: Array[shares mut Player]) changes Player:
    @parallel
    for ref mut p in players:    #$ error[E7010]: `players` holds `shares mut Player` objects, and one object may be in the list twice, so two chunks could change it at once; give each object one owner: write `fn hurt_all(ref mut players: Array[Player]):` instead of line 12, `fn main():` instead of line 18 and `players = [Player(), Player()]` instead of line 19; or guard the fields: write `from std.sync import Lock` at the top, in `Player` `lock: Lock = Lock()`, `hp: int guarded by lock = 10` and `x: int guarded by lock = 0`, and put the loop's body inside `with p.lock:`
        p.hp -= 1
        p.x += p.hp & 1

fn main() changes Player:
    players: Array[shares mut Player] = [shares mut Player(), shares mut Player()]
    hurt_all(ref mut players)
