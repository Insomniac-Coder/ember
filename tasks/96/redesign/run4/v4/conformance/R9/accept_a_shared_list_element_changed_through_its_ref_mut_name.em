#$ test: compile-pass
#$ rules: R9, R8, R5
## R9, R8 (f), (g): the nearest allowed program: the element is changed through its `ref mut` name, and
## the list's other owner is not used while that name is. After `other = shares mut team`, `team` no
## longer owns the list alone, so it is a link (rules section 0) and the change is named,
## `changes Player` (R5, verdict 64).

class Player:
    hp: int = 10

fn main() changes Player:
    team = [Player()]
    other = shares mut team
    boss = ref mut team[0]
    boss.hp = 1
    println(boss.hp)
    println(other.len())
