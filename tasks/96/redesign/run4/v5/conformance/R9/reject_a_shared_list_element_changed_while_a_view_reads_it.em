#$ test: compile-fail
#$ rules: BRW-1
## design rules: R9, R8
## R9, R8 (g): a read-only view of a shared value promises that what it reads does not change through
## any other owner while it is used; today's `Shared` panics on the same overlap while running
## (`tests/conformance/HEAP-5/run_fail_aliased_shared_get_mut.em`), the new model refuses it when
## compiling.

class Player:
    hp: int = 10

fn main():
    team = [Player()]
    other = shares mut team
    boss = team[0]
    other[0].hp = 1               #$ error[E3021]: `other[0].hp = 1` (line 15) changes the player `boss` reads (`boss = team[0]`, line 14), since `other` and `team` own one list, and `boss` is used on line 16; change it after `boss`'s last use, or write `boss = ref mut team[0]` in place of line 14 and change it through `boss`: `boss.hp = 1` in place of line 15
    println(boss.hp)
