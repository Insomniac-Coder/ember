#$ test: compile-pass
#$ rules: RC-7, RC-1, TYP-45
## Design test `accept_a_scene_two_panels_own_and_change.em` of `tasks/96/redesign/run4/v3/conformance/`; where it says so, `owns` is written at an `owns` parameter (`[FN-2a]`) and a `changes` list on a method's first line is left out.
## R9: the nearest allowed program to `reject_refcell_is_not_ember.em`: a scene several objects own and
## change is held `shares mut` by each, and changed by a method whose first line says so (AN-49).

struct Scene:
    entities: Array[int]

    fn add(e: int):
        this.entities.push(owns e)

class Panel:
    scene: shares mut Scene

fn main():
    s = Scene(entities=[])
    left = Panel(scene=shares mut s)
    right = Panel(scene=shares mut s)
    left.scene.add(1)
    right.scene.add(2)
    println(s.entities.len())
