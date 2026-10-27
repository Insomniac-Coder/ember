#$ test: compile-fail
#$ rules: R8, R5, R4
#$ help: change it through `h`: `h += 1`
## R8 (f), R5: `heal`'s first line says it changes `hp`, and `h` is the only name for `p.hp` while it is
## used, so `p.heal()` there changes one place through two names (E3022). Completeness check, G22; the
## allowed neighbour is `accept_a_method_that_changes_hp_while_a_ref_mut_name_of_name_is_used.em`.

class Player:
    name: String
    hp: int = 10

    fn heal() changes hp:
        this.hp += 1

fn main():
    p = Player(name=String("ann"))
    h = ref mut p.hp
    p.heal()    #$ error[E3022]: `p.heal()` (line 18) changes `p.hp` (its first line says `changes hp`), and `h` is the only name for `p.hp` until its last use on line 19; call `p.heal()` below line 19, or change it through `h`: `h += 1`
    h = 5
    println(p.hp)
