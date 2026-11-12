#$ test: compile-pass
#$ rules: RC-7, RC-1, FN-2a
## Design test `accept_a_ref_mut_name_of_a_shares_mut_link_changes_the_player.em` of `tasks/96/redesign/run4/v3/conformance/`; where it says so, `owns` is written at an `owns` parameter (`[FN-2a]`) and a `changes` list on a method's first line is left out.
## R9: "Any `ref mut` name of it ... may change it"; R10 point 1: nothing in `t`'s life can end an
## owner of a `Player`, so `t` costs no count (AN-51: 0 count operations per step where today makes 2).

class Player:
    hp: int = 1000

struct Watcher:
    target: shares mut Player

fn hurt(ref mut ws: Array[Watcher]):
    for i in 0..ws.len():
        t = ref mut ws[i].target      # a name for the watcher's link to its player: no count
        t.hp -= 1

fn main():
    p = Player()
    ws = [Watcher(target=shares mut p)]
    hurt(ref mut ws)
    println(p.hp)
