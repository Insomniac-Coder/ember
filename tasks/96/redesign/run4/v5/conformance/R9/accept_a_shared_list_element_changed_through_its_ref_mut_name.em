#$ test: compile-pass
#$ rules: RC-1, BRW-16, FN-4
## design rules: R9, R8, R5
## R9, R8 (f), (g): the nearest allowed program: the element is changed through its `ref mut` name, and
## the list's other owner is not used while that name is.

class Player:
    hp: int = 10

fn main():
    team = [Player()]
    other = shares mut team
    boss = ref mut team[0]
    boss.hp = 1
    println(boss.hp)
    println(other.len())
