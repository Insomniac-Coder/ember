#$ test: compile-pass
#$ rules: RC-7, RC-1, TYP-45
## Design test `accept_a_field_change_through_a_shares_mut_field.em` of `tasks/96/redesign/run4/v3/conformance/`; where it says so, `owns` is written at an `owns` parameter (`[FN-2a]`) and a `changes` list on a method's first line is left out.
## R9: an owner made with `shares mut`, here a field, may change the shared player,
## through a method that changes it (AN-48 (a)).

class Player:
    hp: int = 100

class Camera:
    target: shares mut Player

    fn shake():
        this.target.hp -= 1

fn main():
    p = Player()
    cam = Camera(target=shares mut p)
    cam.shake()
    println(p.hp)
