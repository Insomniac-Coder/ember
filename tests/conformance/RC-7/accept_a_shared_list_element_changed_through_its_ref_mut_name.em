#$ test: compile-pass
#$ rules: RC-7, RC-1
## Design test `accept_a_shared_list_element_changed_through_its_ref_mut_name.em` of `tasks/96/redesign/run4/v3/conformance/`; where it says so, `owns` is written at an `owns` parameter (`[FN-2a]`) and a `changes` list on a method's first line is left out.
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
