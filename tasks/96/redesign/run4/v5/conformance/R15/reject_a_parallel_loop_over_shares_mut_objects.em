#$ test: compile-fail
#$ rules: R15, R9
## R15: E7010 where chunks may write one place: a list of `shares mut` objects may hold one object twice
## (section 3's message, which gives only "give each player one owner", T-W6 11: guarding the fields
## would send a `shares mut` object to other threads, E7015; with the lines it replaces named, R18;
## AN-74). The nearest allowed program is `accept_a_parallel_loop_over_one_owner_objects.em`.

class Player:
    hp: int = 10
    x: int = 0

fn hurt_all(ref mut players: Array[shares mut Player]):
    @parallel
    for ref mut p in players:    #$ error[E7010]: `players` holds `shares mut Player` objects, and one object may be in the list twice, so two chunks could change it at once; give each player one owner: write `fn hurt_all(ref mut players: Array[Player]):` in place of line 12, and `players = [Player(), Player()]` in place of line 19
        p.hp -= 1
        p.x += p.hp & 1

fn main():
    players: Array[shares mut Player] = [shares mut Player(), shares mut Player()]
    hurt_all(ref mut players)
