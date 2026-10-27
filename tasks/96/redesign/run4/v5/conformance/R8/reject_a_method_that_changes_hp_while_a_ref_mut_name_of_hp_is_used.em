#$ test: compile-fail
#$ rules: BRW-1, BRW-11
#$ help: change it through `h`: `h += 1`
## design rules: R8, R5, R4
## R8 (f), R5: the compiler works out from `heal`'s body that it changes `hp` (verdict 86), and `h` is
## the only name for `p.hp` while it is used, so `p.heal()` there changes one place through two names
## (E3022). Completeness check, G22; the allowed neighbour is
## `accept_a_method_that_changes_hp_while_a_ref_mut_name_of_name_is_used.em`.

class Player:
    name: String
    hp: int = 10

    fn heal():
        this.hp += 1

fn main():
    p = Player(name=String("ann"))
    h = ref mut p.hp
    p.heal()    #$ error[E3022]: `p.heal()` (line 19) changes `p.hp` (through `heal`, `this.hp += 1`, line 14), and `h` is the only name for `p.hp` until its last use on line 20; call `p.heal()` below line 20, or change it through `h`: `h += 1`
    h = 5
    println(p.hp)
